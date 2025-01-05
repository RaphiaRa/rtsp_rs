pub struct ReportBlock<'a> {
    buf: &'a [u8],
}

impl<'a> ReportBlock<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    pub fn ssrc(&self) -> u32 {
        u32::from_be_bytes([self.buf[0], self.buf[1], self.buf[2], self.buf[3]])
    }

    pub fn fraction_lost(&self) -> u8 {
        self.buf[4]
    }

    pub fn packets_lost(&self) -> u32 {
        u32::from_be_bytes([self.buf[5], self.buf[6], self.buf[7], self.buf[8]])
    }

    pub fn highest_sequence(&self) -> u32 {
        u32::from_be_bytes([self.buf[9], self.buf[10], self.buf[11], self.buf[12]])
    }

    pub fn jitter(&self) -> u32 {
        u32::from_be_bytes([self.buf[13], self.buf[14], self.buf[15], self.buf[16]])
    }

    pub fn lsr(&self) -> u32 {
        u32::from_be_bytes([self.buf[17], self.buf[18], self.buf[19], self.buf[20]])
    }

    pub fn dlsr(&self) -> u32 {
        u32::from_be_bytes([self.buf[21], self.buf[22], self.buf[23], self.buf[24]])
    }
}
