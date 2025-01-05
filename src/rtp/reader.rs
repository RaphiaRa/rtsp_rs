use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::Cursor;
use std::net::TcpStream;
use std::vec;
use tokio::io::BufReader;
use tokio::io::ReadBuf;
use tokio::io::Result;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::TcpSocket;
use tokio::net::UdpSocket;

/* RTP Header according to RFC 3550
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|V=2|P|X|  CC   |M|     PT      |       sequence number         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                           timestamp                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           synchronization source (SSRC) identifier            |
+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
|            contributing source (CSRC) identifiers             |
|                             ....                              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

type BufPacket = Vec<u8>;

enum PacketType {
    RTP,
    RTCP,
}

trait GetPacketType {
    fn get_packet_type(&self) -> PacketType;
}

impl GetPacketType for BufPacket {
    fn get_packet_type(&self) -> PacketType {
        let second_byte = self[1];
        return if second_byte >= 200 && second_byte <= 207 {
            PacketType::RTCP
        } else {
            PacketType::RTP
        };
    }
}

#[derive(PartialEq, Eq)]
struct Packet {
    buf: Vec<u8>,
    pub padding: bool,
    pub extension: bool,
    pub marker: bool,
    pub version: u8,
    pub csrc_count: u8,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub data_offset: u32,
}

impl Packet {
    const CSRC_OFFSET: u32 = 12;
    pub fn new(buf: Vec<u8>) -> Result<Self> {
        let first_byte = buf[0];
        let version = first_byte >> 6;
        let padding = (first_byte >> 5) & 0x01 == 1;
        let extension = (first_byte >> 4) & 0x01 == 1;
        let csrc_count = first_byte & 0x0F;
        let second_byte = buf[1];
        let marker = second_byte >> 7 == 1;
        let payload_type = second_byte & 0x7F;
        let sequence_number = u16::from_be_bytes([buf[2], buf[3]]);
        let timestamp = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let ssrc = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let data_offset = Packet::CSRC_OFFSET + (csrc_count * 4) as u32;
        Ok(Self {
            buf,
            version,
            padding,
            extension,
            csrc_count,
            marker,
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            data_offset,
        })
    }

    pub fn data(&self) -> &[u8] {
        if self.padding {
            let padding_len = self.buf[self.buf.len() - 1] as usize;
            &self.buf[self.data_offset as usize..self.buf.len() - padding_len]
        } else {
            &self.buf[self.data_offset as usize..]
        }
    }

    pub fn csrc(&self) -> Vec<u32> {
        let mut csrc = Vec::new();
        for i in 0..self.csrc_count {
            let offset = Packet::CSRC_OFFSET as usize + (i * 4) as usize;
            csrc.push(u32::from_be_bytes([
                self.buf[offset],
                self.buf[offset + 1],
                self.buf[offset + 2],
                self.buf[offset + 3],
            ]));
        }
        csrc
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.sequence_number.cmp(&self.sequence_number))
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.sequence_number.cmp(&self.sequence_number)
    }
}
struct ReorderQueue {
    queue: BinaryHeap<Packet>,
    len: usize,
}

impl ReorderQueue {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, packet: Packet) {
        self.queue.push(packet);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<Packet> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.queue.pop().unwrap())
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn peek(&self) -> Option<&Packet> {
        self.queue.peek()
    }
}
trait ReadBufPacket {
    async fn read_buf_packet(&mut self) -> Result<BufPacket>;
}

impl<T: AsyncRead + Unpin> ReadBufPacket for BufReader<T> {
    async fn read_buf_packet(&mut self) -> Result<BufPacket> {
        // When RTP is transmitted over TCP (or any other stream-oriented protocol),
        // each packet is prefixed with a 2-byte length field.
        let length = self.read_u16().await? as usize;
        let mut buf = vec![0; length];
        self.read_exact(&mut buf).await?;
        Ok(buf)
    }
}

impl ReadBufPacket for UdpSocket {
    async fn read_buf_packet(&mut self) -> Result<BufPacket> {
        let mut buf = vec![0; 3500];
        let (len, _) = self.recv_from(&mut buf).await?;
        buf.truncate(len);
        Ok(buf)
    }
}

pub struct PacketReader<Stream> {
    stream: Stream,
    reorder_queue: ReorderQueue,
    max_reorder_queue_len: usize,
    last_read_sn: u16,
}

impl<Stream: ReadBufPacket> PacketReader<Stream> {
    pub fn new(stream: Stream, max_reorder_queue_len: usize) -> Self {
        Self {
            stream,
            reorder_queue: ReorderQueue::new(),
            max_reorder_queue_len,
            last_read_sn: 0,
        }
    }
    pub fn parse_rtcp(&mut self, packet: BufPacket) {
        // Don't do anything for now
    }

    pub fn push_to_reorder_queue_or_return(&mut self, packet: Packet) -> Option<Packet> {
        if self.last_read_sn == 0 || packet.sequence_number == self.last_read_sn + 1 {
            self.last_read_sn = packet.sequence_number;
            Some(packet)
        } else if packet.sequence_number < self.last_read_sn {
            log::warn!("Packet too old, discarding");
            None
        } else {
            self.reorder_queue.push(packet);
            None
        }
    }

    pub fn parse_rtp(&mut self, packet_buf: BufPacket) -> Option<Packet> {
        let result = Packet::new(packet_buf);
        match result {
            Ok(packet) => self.push_to_reorder_queue_or_return(packet),
            Err(e) => {
                log::error!("Failed to parse RTP packet: {}", e);
                None
            }
        }
    }

    pub fn pop_reorder_queue(&mut self) -> Option<Packet> {
        if let Some(packet) = self.reorder_queue.peek() {
            if packet.sequence_number == self.last_read_sn + 1
                || self.reorder_queue.len() >= self.max_reorder_queue_len
            {
                self.last_read_sn = packet.sequence_number;
                return self.reorder_queue.pop();
            }
        }
        None
    }

    pub fn parse_buf_packet(&mut self, packet_buf: BufPacket) -> Option<Packet> {
        match packet_buf.get_packet_type() {
            PacketType::RTP => self.parse_rtp(packet_buf),
            PacketType::RTCP => self.parse_rtp(packet_buf),
        }
    }

    pub async fn read_packet(&mut self) -> Result<Packet> {
        // Try until we get a packet or an error
        loop {
            let packet = self.pop_reorder_queue();
            match packet {
                Some(packet) => return Ok(packet),
                None => {
                    let packet_buf = self.stream.read_buf_packet().await?;
                    match self.parse_buf_packet(packet_buf) {
                        Some(packet) => return Ok(packet),
                        None => continue,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_new() {
        // Packet buffer with 12 bytes
        // version 2, no padding, no extension, csrc count 0
        // marker 0, payload type 96, sequence number 23, timestamp 0, ssrc 0
        let packet = vec![
            0x80, 0x60, 0x00, 0x17, // version 2, payload type 96, sequence number 23
            0x00, 0x00, 0x00, 0x00, // timestamp 0
            0x00, 0x00, 0x00, 0x00, // ssrc 0
        ];
        let packet = Packet::new(packet).unwrap();
        assert_eq!(packet.version, 2);
        assert_eq!(packet.padding, false);
        assert_eq!(packet.extension, false);
        assert_eq!(packet.csrc_count, 0);
        assert_eq!(packet.marker, false);
        assert_eq!(packet.payload_type, 96);
        assert_eq!(packet.sequence_number, 23);
        assert_eq!(packet.timestamp, 0);
        assert_eq!(packet.ssrc, 0);
    }

    #[tokio::test]
    async fn test_packet_reader() {
        // Create buffer with 5 rtp packets (with 2 byte size prefix)
        // ts and ssrc are set to 0
        // sequence number is set to 23, 25, 27, 24, 26
        let packets = vec![
            vec![
                0x80, 0x60, 0x00, 0x17, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ], // seq 23
            vec![
                0x80, 0x60, 0x00, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ], // seq 25
            vec![
                0x80, 0x60, 0x00, 0x1B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ], // seq 27
            vec![
                0x80, 0x60, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ], // seq 24
            vec![
                0x80, 0x60, 0x00, 0x1A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ], // seq 26
        ];

        let mut buf = Vec::new();
        for packet in packets {
            let length = packet.len() as u16;
            buf.extend_from_slice(&length.to_be_bytes());
            buf.extend_from_slice(&packet);
        }
        let test_reader = tokio_test::io::Builder::new().read(&buf).build();
        let mut packet_reader = PacketReader::new(BufReader::new(test_reader), 5);
        let packet = packet_reader.read_packet().await.unwrap();
        assert_eq!(packet.sequence_number, 23);
        let packet = packet_reader.read_packet().await.unwrap();
        assert_eq!(packet.sequence_number, 24);
        let packet = packet_reader.read_packet().await.unwrap();
        assert_eq!(packet.sequence_number, 25);
        let packet = packet_reader.read_packet().await.unwrap();
        assert_eq!(packet.sequence_number, 26);
        let packet = packet_reader.read_packet().await.unwrap();
        assert_eq!(packet.sequence_number, 27);
        let packet = packet_reader.read_packet().await;
        assert!(packet.is_err());
    }
}
