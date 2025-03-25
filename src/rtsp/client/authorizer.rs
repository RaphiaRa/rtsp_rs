use core::fmt;

use crate::rtsp::protocol::*;
use base64::prelude::*;
use md5::*;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid authorization header")]
    InvalidHeader,
    #[error("Unkown authorization type")]
    UnknownType,
}

type Result<T> = std::result::Result<T, Error>;

type PreparedBuilder<U, H> = RequestBuilder<U, H, VoidBody>;

pub struct Basic {
    auth: String,
}

impl Basic {
    pub fn new(username: &str, password: &str) -> Self {
        let auth = format!("{}:{}", username, password);
        let auth = format!("Basic {}", BASE64_STANDARD.encode(auth.as_bytes()));
        Self { auth }
    }

    fn write(
        &mut self,
        builder: PreparedBuilder<impl fmt::Display, impl CompositeWriter>,
        buf: &mut [u8],
    ) -> std::io::Result<usize> {
        Ok(builder.header("Authorization", &self.auth).write(buf)?)
    }
}

pub struct Digest {
    realm: String,
    nonce: String,
    opaque: String,
    username: String,
    password: String,
    nc: u32,
    rng: StdRng,
}

impl Digest {
    pub fn new(realm: &str, nonce: &str, opaque: &str, username: &str, password: &str) -> Self {
        Self {
            realm: realm.to_string(),
            nonce: nonce.to_string(),
            opaque: opaque.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            nc: 0,
            rng: StdRng::from_os_rng(),
        }
    }

    fn write(
        &mut self,
        url: &Url,
        builder: PreparedBuilder<impl fmt::Display, impl CompositeWriter>,
        buf: &mut [u8],
    ) -> std::io::Result<usize> {
        let nc = self.nc;
        self.nc += 1;
        let cnonce = self.rng.random::<u32>();
        let response = format!(
            "{:x}",
            md5::compute(format!("{}:{}:{}", self.username, self.realm, self.password))
        );
        let auth = format!(
            "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", uri=\"{}\", response=\"{}\", opaque=\"{}\", cnonce=\"{:08x}\", nc={:08x}, qop=auth",
            self.username, self.realm, self.nonce, url.path(), response, self.opaque, cnonce, nc
        );
        Ok(builder.header("Authorization", &auth).write(buf)?)
    }
}

pub struct None {}

impl None {
    pub fn new() -> Self {
        Self {}
    }

    fn write(
        &mut self,
        builder: PreparedBuilder<impl fmt::Display, impl CompositeWriter>,
        buf: &mut [u8],
    ) -> std::io::Result<usize> {
        Ok(builder.write(buf)?)
    }
}

pub enum Authorizer {
    Basic(Basic),
    Digest(Digest),
    None(None),
}

impl Authorizer {
    pub fn write(
        &mut self,
        url: &Url,
        builder: PreparedBuilder<impl fmt::Display, impl CompositeWriter>,
        buf: &mut [u8],
    ) -> std::io::Result<usize> {
        match self {
            Authorizer::Basic(basic) => basic.write(builder, buf),
            Authorizer::Digest(digest) => digest.write(url, builder, buf),
            Authorizer::None(none) => none.write(builder, buf),
        }
    }

    pub fn new(user: &str, pass: &str, www_auth: &str) -> Result<Self>{
        let mut iter = www_auth.splitn(2, ' ');
        let auth_type = iter.next().ok_or(Error::InvalidHeader)?;
        let auth_data = iter.next().ok_or(Error::InvalidHeader)?;
        match auth_type {
            "Basic" => Ok(Authorizer::Basic(Basic::new(user, pass))),
            "Digest" => {
                let mut realm = None;
                let mut nonce = None;
                let mut opaque = None;
                for item in auth_data.split(',') {
                    let mut iter = item.splitn(2, '=');
                    let key = iter.next().ok_or(Error::InvalidHeader)?;
                    let value = iter.next().ok_or(Error::InvalidHeader)?;
                    let value = value.trim();
                    let value = value.trim_matches('\"');
                    match key {
                        "realm" => realm = Some(value.to_string()),
                        "nonce" => nonce = Some(value.to_string()),
                        "opaque" => opaque = Some(value.to_string()),
                        _ => {}
                    }
                }
                Ok(Authorizer::Digest(Digest::new(
                    &realm.unwrap(),
                    &nonce.unwrap(),
                    &opaque.unwrap(),
                    user,
                    pass,
                )))
            }
            _ => Err(Error::UnknownType),
        }
    }
}

impl Default for Authorizer {
    fn default() -> Self {
        Authorizer::None(None::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_authorizer() {
        let mut authorizer = Authorizer::Basic(Basic::new("user", "pass"));
        let url = Url::parse("rtsp://localhost:554/test").unwrap();
        let mut builder = RequestBuilder::new().url(&url).method(Method::Options);
        let mut buf = [0; 1024];
        let n = authorizer.write(&url, &builder, &mut buf).unwrap();
        assert_eq!(
            &buf[..n],
            b"OPTIONS rtsp://localhost:554/test RTSP/1.0\r\n
            Authorization: Basic dXNlcjpwYXNz\r\n
            \r\n"
        );
    }
}


