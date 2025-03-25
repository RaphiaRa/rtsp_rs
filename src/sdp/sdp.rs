use std::{convert::TryFrom};
use thiserror::Error;

#[derive(Error, Debug)]
pub struct Sdp {
    description: String,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid SDP format")]
    InvalidFormat,
}

impl TryFrom<&str> for Sdp {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Sdp {
            description: value.to_string(),
        })
    }
}

impl std::fmt::Display for Sdp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}
