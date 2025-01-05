pub struct SDESItem<'a> {
    buf: &'a [u8],
}

impl<'a> SDESItem<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    pub fn str(&self) -> &str {
        let length = self.buf[1] as usize;
        std::str::from_utf8(&self.buf[2..length + 2]).unwrap()
    }
}
