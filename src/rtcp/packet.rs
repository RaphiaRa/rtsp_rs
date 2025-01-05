use super::{Header, SenderReport};
use std::io;
pub struct Packet<'a> {
    pub buf: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, io::Error> {
        if buf.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid RTCP packet",
            ));
        }
        Ok(Self { buf })
    }

    pub fn header(&self) -> Header {
        Header::new(&self.buf[0..4]).unwrap()
    }

    pub fn to_sender_report(&self) -> Result<SenderReport, io::Error> {
        SenderReport::new(&self.buf)
    }
}

/// RTCP Compound Packet
/// Format according to RFC 3550
/// if encrypted: random 32-bit integer
/// |
/// |[--------- packet --------][---------- packet ----------][-packet-]
/// |
/// |                receiver            chunk        chunk
/// V                reports           item  item   item  item
/// --------------------------------------------------------------------
/// R[SR #sendinfo #site1#site2][SDES #CNAME PHONE #CNAME LOC][BYE##why]
/// --------------------------------------------------------------------
/// |                                                                  |
/// |<-----------------------  compound packet ----------------------->|
/// |<--------------------------  UDP packet ------------------------->|
pub struct CompoundPacket {
    pub payload: Vec<u8>,
}

struct CompoundPacketIterator<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for CompoundPacketIterator<'a> {
    type Item = Packet<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.buf.len() {
            return None;
        }
        let packet = Packet::new(&self.buf[self.offset..]);
        match packet {
            Ok(p) => {
                self.offset += (1 + p.header().length() as usize) * 4;
                Some(p)
            }
            Err(_) => {
                // TODO: log error
                self.offset = self.buf.len();
                None
            }
        }
    }
}

impl CompoundPacket {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    pub fn iter(&self) -> CompoundPacketIterator {
        CompoundPacketIterator {
            buf: &self.payload,
            offset: 0,
        }
    }
}
