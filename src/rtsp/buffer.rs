use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Not enough buffer space")]
    NotEnoughSpace,
}

type Result<T> = std::result::Result<T, BufferError>;

pub struct Buffer {
    data: Vec<u8>,
    max_capacity: usize,
    read_pos: usize,
    write_pos: usize,
}

impl Buffer {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            data: Vec::new(),
            max_capacity,
            read_pos: 0,
            write_pos: 0,
        }
    }

    pub fn get_read_slice(&mut self) -> &[u8] {
        let slice = &self.data[self.read_pos..self.write_pos];
        slice
    }

    pub fn notify_read(&mut self, n: usize) {
        self.read_pos += n;
        if self.read_pos == self.write_pos {
            self.read_pos = 0;
            self.write_pos = 0;
        }
    }

    pub fn get_write_slice(&mut self, n: usize) -> Result<&mut [u8]> {
        if self.write_pos + n > self.data.len() {
            if n <= self.read_pos {
                self.data.copy_within(self.read_pos..self.write_pos, 0);
                self.write_pos -= self.read_pos;
                self.read_pos = 0;
            } else if self.write_pos + n <= self.max_capacity {
                self.data.resize(self.write_pos + n, 0);
            } else {
                return Err(BufferError::NotEnoughSpace);
            }
        }
        let slice = &mut self.data[self.write_pos..];
        Ok(slice)
    }

    pub fn notify_write(&mut self, n: usize) {
        self.write_pos += n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer() {
        let mut buffer = Buffer::new(10);
            let slice = buffer.get_write_slice(5).unwrap();
            slice.copy_from_slice(&[1, 2, 3, 4, 5]);
            buffer.notify_write(5);
            let slice = buffer.get_write_slice(5).unwrap();
            slice.copy_from_slice(&[6, 7, 8, 9, 10]);
            buffer.notify_write(5);
            let slice = buffer.get_read_slice();
            assert_eq!(slice, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            buffer.notify_read(10);
            let slice = buffer.get_write_slice(5).unwrap();
            slice[..5].copy_from_slice(&[11, 12, 13, 14, 15]);
            buffer.notify_write(5);
            let slice = buffer.get_read_slice();
            assert_eq!(slice, &[11, 12, 13, 14, 15]);
    }
}
