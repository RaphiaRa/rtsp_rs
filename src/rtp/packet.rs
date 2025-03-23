use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Buffer too short to be an RTP packet")]
    BufferTooShort,
}

pub type Result<T> = std::result::Result<T, Error>;

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

#[derive(PartialEq, Eq)]

pub struct Packet {
    buf: Vec<u8>,
}

impl Packet {
    const CSRC_OFFSET: u32 = 12;
    pub fn new(buf: Vec<u8>) -> Result<Packet> {
        let packet = Packet { buf };
        if packet.len() < 12 || packet.len() < packet.data_offset() as usize {
            return Err(Error::BufferTooShort);
        }
        Ok(packet)
    }

    pub fn version(&self) -> u8 {
        self.buf[0] >> 6
    }

    pub fn padding(&self) -> bool {
        (self.buf[0] >> 5) & 0x01 == 1
    }

    pub fn extension(&self) -> bool {
        (self.buf[0] >> 4) & 0x01 == 1
    }

    pub fn csrc_count(&self) -> u8 {
        self.buf[0] & 0x0F
    }

    pub fn marker(&self) -> bool {
        self.buf[1] >> 7 == 1
    }

    pub fn payload_type(&self) -> u8 {
        self.buf[1] & 0x7F
    }

    pub fn sequence_number(&self) -> u16 {
        u16::from_be_bytes([self.buf[2], self.buf[3]])
    }

    pub fn timestamp(&self) -> u32 {
        u32::from_be_bytes([self.buf[4], self.buf[5], self.buf[6], self.buf[7]])
    }

    pub fn ssrc(&self) -> u32 {
        u32::from_be_bytes([self.buf[8], self.buf[9], self.buf[10], self.buf[11]])
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    fn data_offset(&self) -> u32 {
        Packet::CSRC_OFFSET + (self.csrc_count() * 4) as u32
    }

    pub fn data(&self) -> &[u8] {
        if self.padding() {
            let padding_len = self.buf[self.buf.len() - 1] as usize;
            &self.buf[self.data_offset() as usize..self.buf.len() - padding_len]
        } else {
            &self.buf[self.data_offset() as usize..]
        }
    }

    pub fn csrc(&self) -> Vec<u32> {
        let mut csrc = Vec::new();
        for i in 0..self.csrc_count() {
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
        Some(other.sequence_number().cmp(&self.sequence_number()))
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.sequence_number().cmp(&self.sequence_number())
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
        assert_eq!(packet.version(), 2);
        assert_eq!(packet.padding(), false);
        assert_eq!(packet.extension(), false);
        assert_eq!(packet.csrc_count(), 0);
        assert_eq!(packet.marker(), false);
        assert_eq!(packet.payload_type(), 96);
        assert_eq!(packet.sequence_number(), 23);
        assert_eq!(packet.timestamp(), 0);
        assert_eq!(packet.ssrc(), 0);
        assert_eq!(packet.len(), 12);
        assert_eq!(packet.data().len(), 0);
    }
}
