use std::fmt;
use std::str::FromStr;
use thiserror::Error;

pub enum Method {
    OPTIONS,
    DESCRIBE,
    SETUP,
    PLAY,
    TEARDOWN,
}
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::OPTIONS => write!(f, "OPTIONS"),
            Method::DESCRIBE => write!(f, "DESCRIBE"),
            Method::SETUP => write!(f, "SETUP"),
            Method::PLAY => write!(f, "PLAY"),
            Method::TEARDOWN => write!(f, "TEARDOWN"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseMethodError {
    #[error("Invalid method")]
    InvalidMethod,
}
impl FromStr for Method {
    type Err = ParseMethodError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OPTIONS" => Ok(Method::OPTIONS),
            "DESCRIBE" => Ok(Method::DESCRIBE),
            "SETUP" => Ok(Method::SETUP),
            "PLAY" => Ok(Method::PLAY),
            "TEARDOWN" => Ok(Method::TEARDOWN),
            _ => Err(ParseMethodError::InvalidMethod),
        }
    }
}
