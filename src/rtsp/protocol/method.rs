use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Options,
    Describe,
    Setup,
    Play,
    Teardown,
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::Options => "OPTIONS",
            Method::Describe => "DESCRIBE",
            Method::Setup => "SETUP",
            Method::Play => "PLAY",
            Method::Teardown => "TEARDOWN",
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::Options => write!(f, "OPTIONS"),
            Method::Describe => write!(f, "DESCRIBE"),
            Method::Setup => write!(f, "SETUP"),
            Method::Play => write!(f, "PLAY"),
            Method::Teardown => write!(f, "TEARDOWN"),
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
            "OPTIONS" => Ok(Method::Options),
            "DESCRIBE" => Ok(Method::Describe),
            "SETUP" => Ok(Method::Setup),
            "PLAY" => Ok(Method::Play),
            "TEARDOWN" => Ok(Method::Teardown),
            _ => Err(ParseMethodError::InvalidMethod),
        }
    }
}
