use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[error("Expected format: major.minor")]
    Format,
}

impl FromStr for Version {
    type Err = ParseVersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('.');
        let major = iter.next().ok_or(ParseVersionError::Format)?.parse::<u8>()?;
        let minor = iter.next().ok_or(ParseVersionError::Format)?.parse::<u8>()?;
        if iter.next().is_some() {
            return Err(ParseVersionError::Format);
        }
        Ok(Version(major, minor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let version = "1.0".parse::<Version>().unwrap();
        assert_eq!(version, Version::new(1, 0));
    }

    #[test]
    fn test_parse_version_missing_token() {
        let result = "1".parse::<Version>();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseVersionError::Format));
    }

    #[test]
    fn test_parse_version_invalid_version() {
        let result = "1.0.0".parse::<Version>();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseVersionError::Format));
    }

    #[test]
    fn test_parse_version_invalid_digit() {
        let result = "1.a".parse::<Version>();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseVersionError::ParseInt(_)));
    }
}
