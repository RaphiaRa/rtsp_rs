use super::Method;
use super::Version;
use crate::rtcp::Header;
use std::fmt;
use std::io;
use std::io::Write;
use std::num::ParseIntError;
use std::str::FromStr;
use url::Url;

enum State {
    BuildHeaders,
    BuildBody,
}

type RequestBuf = Vec<u8>;

pub struct RequestBuilder {
    buffer: Vec<u8>,
    method: Method,
    url: Url,
    version: Version,
}

impl RequestBuilder {
    pub fn new(url: Url, method: Method) -> Self {
        Self {
            buffer: Vec::with_capacity(1024),
            method: method,
            url,
            version: Version::new(1, 0),
        }
    }

    pub fn version(&mut self, version: Version) -> &mut Self {
        self.version = version;
        self
    }

    pub fn header<T: fmt::Display>(&mut self, key: &str, value: T) -> HeaderBuilder {
        write!(self.buffer, "{} {} RTSP/{}\r\n", self.method, self.url, self.version,).unwrap();
        write!(self.buffer, "{}: {}\r\n", key, value).unwrap();
        HeaderBuilder {
            buffer: std::mem::replace(&mut self.buffer, Vec::new()),
        }
    }
}

pub struct HeaderBuilder {
    buffer: Vec<u8>,
}

impl HeaderBuilder {
    pub fn header<T: fmt::Display>(&mut self, key: &str, value: T) -> &mut Self {
        write!(self.buffer, "{}: {}\r\n", key, value).unwrap();
        self
    }

    pub fn body(&mut self, body: &[u8]) -> FinalBuilder {
        write!(self.buffer, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
        self.buffer.extend_from_slice(body);
        FinalBuilder {
            buffer: std::mem::replace(&mut self.buffer, Vec::new()),
        }
    }

    pub fn builder(&mut self) -> HeaderBuilder {
        HeaderBuilder {
            buffer: std::mem::replace(&mut self.buffer, Vec::new()),
        }
    }

    pub fn result(&mut self) -> RequestBuf {
        write!(self.buffer, "Content-Length: 0\r\n\r\n").unwrap();
        std::mem::replace(&mut self.buffer, Vec::new())
    }
}

pub struct FinalBuilder {
    buffer: Vec<u8>,
}

impl FinalBuilder {
    pub fn result(&mut self) -> RequestBuf {
        std::mem::replace(&mut self.buffer, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_buf_builder() {
        let mut builder = RequestBuilder::new(Url::parse("rtsp://test.com").unwrap(), Method::DESCRIBE);
        let buf = builder
            .version(Version::new(1, 0))
            .header("CSeq", 1)
            .header("Accept", "application/sdp")
            .header("User-Agent", "apex")
            .header("Session", "123456")
            .body(b"")
            .result();
        assert_eq!(
            buf,
            b"DESCRIBE rtsp://test.com RTSP/1.0\r\nCSeq: 1\r\nAccept: application/sdp\r\nUser-Agent: apex\r\nSession: 123456\r\nContent-Length: 0\r\n\r\n"
        );
    }
}
