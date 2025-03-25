use super::Method;
use super::Version;
use std::fmt;
use std::io::Write;
use url::Url;

pub type BuildError = std::io::Error;
pub type Result<T> = std::result::Result<T, BuildError>;

pub trait RequestWriter {
    fn write(&self, buf: &mut [u8]) -> Result<usize>;
}

pub struct VoidHeader {}

impl RequestWriter for VoidHeader {
    fn write(&self, buf: &mut [u8]) -> Result<usize> {
        write!(&mut buf[..], "\r\n")?;
        Ok(2)
    }
}

pub struct HeaderWriter<'a, V> {
    name: &'a str,
    value: V,
}

impl<V: fmt::Display> RequestWriter for HeaderWriter<'_, V> {
    fn write(&self, buf: &mut [u8]) -> Result<usize> {
        let mut cursor = std::io::Cursor::new(buf);
        write!(cursor, "{}: {}\r\n", self.name, self.value)?;
        Ok(cursor.position() as usize)
    }
}

pub struct Composite<A, B> {
    a: A,
    b: B,
}

pub trait CompositeWriter {
    fn write_composite(&self, buf: &mut [u8]) -> Result<usize>;
}

impl<B: RequestWriter> CompositeWriter for Composite<VoidHeader, B> {
    fn write_composite(&self, buf: &mut [u8]) -> Result<usize> {
        // ignore void header
        Ok(self.b.write(&mut buf[..])?)
    }
}

impl<A: CompositeWriter, B: RequestWriter> CompositeWriter for Composite<A, B> {
    fn write_composite(&self, buf: &mut [u8]) -> Result<usize> {
        let pos = self.a.write_composite(buf)?;
        Ok(pos + self.b.write(&mut buf[pos..])?)
    }
}

impl<A: CompositeWriter, B: RequestWriter> RequestWriter for Composite<A, B> {
    fn write(&self, buf: &mut [u8]) -> Result<usize> {
        let pos = self.write_composite(buf)?;
        Ok(pos + VoidHeader {}.write(&mut buf[pos..])?)
    }
}

pub struct VoidBody {}

impl RequestWriter for VoidBody {
    fn write(&self, _buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }
}
pub struct BodyWriter<B> {
    body: B,
}

impl<B: fmt::Display> RequestWriter for BodyWriter<B> {
    fn write(&self, buf: &mut [u8]) -> Result<usize> {
        let mut cursor = std::io::Cursor::new(buf);
        write!(cursor, "{}", self.body)?;
        Ok(cursor.position() as usize)
    }
}

pub struct VoidUrl {}

impl fmt::Display for VoidUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rtsp://")
    }
}

#[derive(Debug, Clone)]
pub struct RequestBuilder<U, H, B> {
    method: Method,
    version: Version,
    url: U,
    headers: H,
    body: B,
}

impl<U: fmt::Display, H: CompositeWriter, B: RequestWriter> RequestWriter for RequestBuilder<U, H, B> {
    fn write(&self, buf: &mut [u8]) -> Result<usize> {
        let mut cursor = std::io::Cursor::new(buf);
        write!(cursor, "{} {} RTSP/{}\r\n", self.method, self.url, self.version)?;
        let mut pos = cursor.position() as usize;
        pos += self.headers.write_composite(&mut cursor.get_mut()[pos..])?;
        pos += self.body.write(&mut cursor.get_mut()[pos..])?;
        Ok(pos)
    }
}

impl<U: fmt::Display, B: RequestWriter> RequestWriter for RequestBuilder<U, VoidHeader, B> {
    fn write(&self, buf: &mut [u8]) -> Result<usize> {
        let mut cursor = std::io::Cursor::new(buf);
        write!(cursor, "{} {} RTSP/{}\r\n", self.method, self.url, self.version)?;
        let mut pos = cursor.position() as usize;
        pos += self.headers.write(&mut cursor.get_mut()[pos..])?;
        pos += self.body.write(&mut cursor.get_mut()[pos..])?;
        Ok(pos)
    }
}

impl RequestBuilder<VoidUrl, VoidHeader, VoidBody> {
    pub fn new() -> Self {
        Self {
            method: Method::Options,
            url: VoidUrl {},
            version: Version::new(1, 0),
            headers: VoidHeader {},
            body: VoidBody {},
        }
    }
}

impl<U, H, B> RequestBuilder<U, H, B> {
    pub fn version(self, version: Version) -> Self {
        Self { version, ..self }
    }

    pub fn method(self, method: Method) -> Self {
        Self { method, ..self }
    }
}

impl<H, B> RequestBuilder<VoidUrl, H, B> {
    pub fn url<'a>(self, url: &'a Url) -> RequestBuilder<&'a Url, H, B> {
        RequestBuilder {
            method: self.method,
            url,
            version: self.version,
            headers: self.headers,
            body: self.body,
        }
    }
}

impl<U> RequestBuilder<U, VoidHeader, VoidBody> {
    pub fn header<'a, V: fmt::Display>(
        self,
        name: &'a str,
        value: V,
    ) -> RequestBuilder<U, Composite<VoidHeader, HeaderWriter<'a, V>>, VoidBody> {
        RequestBuilder {
            method: self.method,
            url: self.url,
            version: self.version,
            headers: Composite {
                a: self.headers,
                b: HeaderWriter { name, value },
            },
            body: self.body,
        }
    }
}
impl<U, H: CompositeWriter> RequestBuilder<U, H, VoidBody> {
    pub fn header<'a, V: fmt::Display>(
        self,
        name: &'a str,
        value: V,
    ) -> RequestBuilder<U, Composite<H, HeaderWriter<'a, V>>, VoidBody> {
        RequestBuilder {
            method: self.method,
            url: self.url,
            version: self.version,
            headers: Composite {
                a: self.headers,
                b: HeaderWriter { name, value },
            },
            body: self.body,
        }
    }

    pub fn body<'a>(
        self,
        body: &'a str,
    ) -> RequestBuilder<U, Composite<H, HeaderWriter<'static, usize>>, BodyWriter<&'a str>> {
        let s = self.header("Content-Length", body.len());
        RequestBuilder {
            method: s.method,
            url: s.url,
            version: s.version,
            headers: s.headers,
            body: BodyWriter { body },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let mut buf = [0u8; 128];
        let n = RequestBuilder::new()
            .url(&Url::parse("rtsp://test.com").unwrap())
            .method(Method::Describe)
            .version(Version::new(1, 0))
            .header("CSeq", 1)
            .header("User-Agent", "test")
            .body("test")
            .write(&mut buf)
            .unwrap();
        assert_eq!(
            std::str::from_utf8(&buf[..n]).unwrap(),
            "DESCRIBE rtsp://test.com RTSP/1.0\r\nCSeq: 1\r\nUser-Agent: test\r\nContent-Length: 4\r\n\r\ntest"
        );
    }
    #[test]
    fn test_request_builder_insufficient_buffer() {
        let mut buf = [0u8; 10];
        let n = RequestBuilder::new()
            .url(&Url::parse("rtsp://test.com").unwrap())
            .method(Method::Describe)
            .version(Version::new(1, 0))
            .header("CSeq", 1)
            .header("User-Agent", "test")
            .body("test")
            .write(&mut buf);
        assert!(n.is_err());
    }
}
