use std::convert::TryFrom;
use thiserror::Error;

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
