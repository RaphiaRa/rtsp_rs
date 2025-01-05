use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Header<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> Header<'a> {
    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }
}

impl<'a> fmt::Display for Header<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}\r\n", self.name, self.value)
    }
}

#[derive(Error, Debug)]
pub enum ParseHeaderError {
    #[error("Invalid header format")]
    InvalidFormat,
    #[error("Invalid header name")]
    InvalidName,
}

type Result<T> = std::result::Result<T, ParseHeaderError>;

fn verify_header_name(name: &str) -> Result<()> {
    if name.len() > 0 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        Ok(())
    } else {
        Err(ParseHeaderError::InvalidName)
    }
}

fn verify_header_value(value: &str) -> Result<()> {
    if value.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
        Ok(())
    } else {
        Err(ParseHeaderError::InvalidFormat)
    }
}

impl<'a> TryFrom<&'a str> for Header<'a> {
    type Error = ParseHeaderError;

    fn try_from(value: &'a str) -> Result<Self> {
        let mut parts = value.splitn(2, ':');
        let name = parts.next().ok_or(ParseHeaderError::InvalidFormat)?;
        verify_header_name(name)?;
        let value = parts.next().ok_or(ParseHeaderError::InvalidFormat)?;
        verify_header_value(value)?;
        Ok(Header::new(name, value.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let header = Header::try_from("Content-Length: 123").unwrap();
        assert_eq!(header.name, "Content-Length");
        assert_eq!(header.value, "123");
    }

    #[test]
    fn test_parse_header_missing_colon() {
        let result = Header::try_from("Content-Length 123");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_header_space_before_colon() {
        let result = Header::try_from("Content-Length : 123");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_header_empty_value() {
        let header = Header::try_from("Content-Length:").unwrap();
        assert_eq!(header.name, "Content-Length");
        assert_eq!(header.value, "");
    }
}
