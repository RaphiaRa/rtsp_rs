use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Version(u8, u8);

impl Version {
    pub fn new(major: u8, minor: u8) -> Self {
        Version(major, minor)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

#[derive(Debug, Error)]
pub enum ParseVersionError {
    #[error("Failed to parse version digits")]
    ParseInt(#[from] ParseIntError),
    #[error("Invalid version format")]
    Format(#[from] fmt::Error),
}

impl FromStr for Version {
    type Err = ParseVersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('.');
        let major = iter.next().ok_or(fmt::Error)?.parse::<u8>()?;
        let minor = iter.next().ok_or(fmt::Error)?.parse::<u8>()?;
        Ok(Version(major, minor))
    }
}

#[derive(Debug, PartialEq)]
pub struct Protocol {
    version: Version,
}

impl Protocol {
    pub fn new(version: Version) -> Self {
        Protocol { version }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RTSP/{}", self.version)
    }
}

#[derive(Debug, Error)]
pub enum ParseProtocolError {
    #[error("Unexpected token")]
    UnexpectedToken,
    #[error("Failed to parse version {0}")]
    ParseVersion(#[from] ParseVersionError),
}

impl FromStr for Protocol {
    type Err = ParseProtocolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('/');
        let token = iter.next().ok_or(ParseProtocolError::UnexpectedToken)?;
        (token == "RTSP")
            .then_some(())
            .ok_or(ParseProtocolError::UnexpectedToken)?;
        let version = iter.next().ok_or(ParseProtocolError::UnexpectedToken)?.parse()?;
        Ok(Protocol::new(version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let version = "RTSP/1.0".parse::<Protocol>().unwrap();
        assert_eq!(version.version, Version::new(1, 0));
    }

    #[test]
    fn test_parse_version_missing_token() {
        let result = "1.0".parse::<Protocol>();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseProtocolError::UnexpectedToken));
    }

    #[test]
    fn test_parse_version_invalid_version() {
        let result = "RTSP/1".parse::<Protocol>();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseProtocolError::ParseVersion(_)));
    }
}
