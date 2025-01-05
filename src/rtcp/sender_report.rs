use super::{Header, ReportBlock};
use std::io;

pub struct SenderReport<'a> {
    buf: &'a [u8],
}

impl<'a> SenderReport<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, io::Error> {
        if buf.len() < 24 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid RTCP Sender Report",
            ));
        }
        Ok(Self { buf })
    }

    pub fn header(&self) -> Header {
        Header::new(&self.buf[0..4]).unwrap()
    }

    pub fn ssrc(&self) -> u32 {
        u32::from_be_bytes([self.buf[4], self.buf[5], self.buf[6], self.buf[7]])
    }

    pub fn ntp_timestamp(&self) -> u64 {
        u64::from_be_bytes([
            self.buf[8],
            self.buf[9],
            self.buf[10],
            self.buf[11],
            self.buf[12],
            self.buf[13],
            self.buf[14],
            self.buf[15],
        ])
    }

    pub fn rtp_ts(&self) -> u32 {
        u32::from_be_bytes([self.buf[16], self.buf[17], self.buf[18], self.buf[19]])
    }

    pub fn packets_sent(&self) -> u32 {
        u32::from_be_bytes([self.buf[20], self.buf[21], self.buf[22], self.buf[23]])
    }

    pub fn octets_sent(&self) -> u32 {
        u32::from_be_bytes([self.buf[24], self.buf[25], self.buf[26], self.buf[27]])
    }

    pub fn report_blocks(&self) -> Vec<ReportBlock> {
        let mut blocks = Vec::new();
        let mut offset = 28;
        for _ in 0..self.header().count() {
            blocks.push(ReportBlock::new(&self.buf[offset..offset + 24]));
            offset += 24;
        }
        blocks
    }

    pub fn size(&self) -> usize {
        28 + self.header().count() * 24
    }
}
