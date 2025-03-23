use tokio::io::{AsyncReadExt, Result};

pub struct AsyncCursor<R> {
    reader: R,
    position: usize,
}

impl<R: AsyncReadExt + Unpin> AsyncCursor<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            position: 0,
        }
    }

    pub async fn read_u8(&mut self) -> Result<u8> {
        let v = self.reader.read_u8().await?;
        self.position += 1;
        Ok(v)
    }

    pub async fn read_u16(&mut self) -> Result<u16> {
        let v = self.reader.read_u16().await?;
        self.position += 2;
        Ok(v)
    }

    pub async fn read_u32(&mut self) -> Result<u32> {
        let v = self.reader.read_u32().await?;
        self.position += 4;
        Ok(v)
    }

    pub fn position(&self) -> usize {
        self.position
    }
}
