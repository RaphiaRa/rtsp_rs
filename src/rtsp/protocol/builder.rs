use super::Method;
use super::Version;
use std::fmt;
use std::io::Write;
use url::Url;
pub type Error = std::io::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub struct NoHeader {}

impl fmt::Display for NoHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

pub struct Header<'a, V> {
    name: &'a str,
    value: V,
}

impl<V: fmt::Display> fmt::Display for Header<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}\r\n", self.name, self.value)
    }
}

pub struct Composite<A, B> {
    a: A,
    b: B,
}

impl<A: fmt::Display, B: fmt::Display> fmt::Display for Composite<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.a, self.b)
    }
}

pub struct NoBody {}

impl fmt::Display for NoBody {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

pub struct NoUrl {}

impl fmt::Display for NoUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rtsp://");
        Ok(())
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

impl<U: fmt::Display, H: fmt::Display, B: fmt::Display> fmt::Display for RequestBuilder<U, H, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} RTSP/{}\r\n{}\r\n{}",
            self.method, self.url, self.version, self.headers, self.body
        )
    }
}

impl RequestBuilder<NoUrl, NoHeader, NoBody> {
    pub fn new() -> Self {
        Self {
            method: Method::Options,
            url: NoUrl {},
            version: Version::new(1, 0),
            headers: NoHeader {},
            body: NoBody {},
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

impl<H, B> RequestBuilder<NoUrl, H, B> {
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

impl<U, H> RequestBuilder<U, H, NoBody> {
    pub fn header<'a, V: fmt::Display>(
        self,
        name: &'a str,
        value: V,
    ) -> RequestBuilder<U, Composite<H, Header<'a, V>>, NoBody> {
        RequestBuilder {
            method: self.method,
            url: self.url,
            version: self.version,
            headers: Composite {
                a: self.headers,
                b: Header { name, value },
            },
            body: self.body,
        }
    }

    pub fn body<'a>(self, body: &'a str) -> RequestBuilder<U, Composite<H, Header<'static, usize>>, &'a str> {
        let builder = self.header("Content-Length", body.len());
        RequestBuilder {
            method: builder.method,
            url: builder.url,
            version: builder.version,
            headers: builder.headers,
            body,
        }
    }
}

pub trait Serialize {
    fn serialize(&self, buf: &mut [u8]) -> Result<usize>;
}

impl<T: fmt::Display> Serialize for T {
    fn serialize(&self, buf: &mut [u8]) -> Result<usize> {
        let mut cursor = std::io::Cursor::new(buf);
        write!(cursor, "{}", self)?;
        Ok(cursor.position() as usize)
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
            .serialize(&mut buf)
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
            .serialize(&mut buf);
        assert!(n.is_err());
    }
}
