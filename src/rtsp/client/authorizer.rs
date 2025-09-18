use crate::rtsp::protocol::*;
use base64::prelude::*;
use digest_auth::{AuthContext, HttpMethod, WwwAuthenticateHeader};
use std::borrow::Cow;

use std::option::Option;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid authorization header")]
    InvalidHeader,
    #[error("Unkown authorization type")]
    UnknownType,
    #[error(transparent)]
    DigestAuthError(#[from] digest_auth::Error),
}

type Result<T> = std::result::Result<T, Error>;

type Answer = String;

pub struct Basic {
    auth: String,
}

impl Basic {
    pub fn new(username: &str, password: &str) -> Self {
        let auth = format!("{}:{}", username, password);
        let auth = format!("Basic {}", BASE64_STANDARD.encode(auth.as_bytes()));
        Self { auth }
    }

    fn answer(&mut self) -> Result<Answer> {
        Ok(self.auth.clone())
    }
}

pub struct Digest {
    username: String,
    password: String,
    www_authenticate: WwwAuthenticateHeader,
}

impl Digest {
    pub fn new(username: &str, password: &str, www_authenticate: &str) -> Result<Self> {
        Ok(Self {
            username: username.to_string(),
            password: password.to_string(),
            www_authenticate: WwwAuthenticateHeader::parse(www_authenticate)?,
        })
    }

    fn answer(&mut self, method: Method, url: &Url) -> Result<Answer> {
        let context = AuthContext::new_with_method(
            &self.username,
            &self.password,
            url.path().to_string(),
            Option::<&'_ [u8]>::None,
            HttpMethod(Cow::Borrowed(method.as_str())),
        );
        Ok(self.www_authenticate.respond(&context)?.to_string())
    }
}

pub enum Authorizer {
    Basic(Basic),
    Digest(Digest),
}

impl Authorizer {
    pub fn answer(&mut self, method: Method, url: &Url) -> Result<Answer> {
        match self {
            Authorizer::Basic(basic) => basic.answer(),
            Authorizer::Digest(digest) => digest.answer(method, url),
        }
    }

    pub fn new(user: &str, pass: &str, www_auth: &str) -> Result<Self> {
        let mut iter = www_auth.splitn(2, ' ');
        let auth_type = iter.next().ok_or(Error::InvalidHeader)?;
        let auth_data = iter.next().ok_or(Error::InvalidHeader)?;
        match auth_type {
            "Basic" => Ok(Authorizer::Basic(Basic::new(user, pass))),
            "Digest" => Ok(Authorizer::Digest(Digest::new(user, pass, auth_data)?)),
            _ => Err(Error::UnknownType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_authorizer() {
        let mut authorizer = Authorizer::Basic(Basic::new("user", "pass"));
        let url = Url::parse("rtsp://localhost:554/test").unwrap();
        let answer = authorizer.answer(Method::Options, &url).unwrap();
        assert_eq!(answer, "Basic dXNlcjpwYXNz");
    }
}
