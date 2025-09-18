use super::*;
use crate::rtsp::protocol::*;
use crate::sdp;

use std::fmt;

use thiserror::Error;
use tokio::sync::oneshot;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ParseSdp(#[from] sdp::ParseError),
    #[error("Unexpected status code: {0}")]
    UnexpectedStatus(Status),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Cancelled")]
    Cancelled,
    #[error("Bad response")]
    BadResponse,
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Describe {
    url: url::Url,
    tx: oneshot::Sender<Result<sdp::Sdp>>,
}

impl Describe {
    pub fn handle_response(self, status: Status, _headers: &[Header], body: &str) {
        if status != Status::OK {
            let _ = self.tx.send(Err(Error::UnexpectedStatus(status)));
        } else {
            match sdp::Sdp::try_from(body) {
                Ok(sdp) => self.tx.send(Ok(sdp)),
                Err(e) => self.tx.send(Err(Error::ParseSdp(e))),
            };
        }
    }

    pub fn url(&self) -> &url::Url {
        &self.url
    }

    pub fn method(&self) -> Method {
        Method::Describe
    }

    pub fn cancel(self, e: Error) {
        let _ = self.tx.send(Err(e));
    }
}

impl Describe {
    pub fn new(url: url::Url, tx: oneshot::Sender<Result<sdp::Sdp>>) -> Self {
        Self { url, tx }
    }
}

pub enum Command {
    Describe(Describe),
}

impl Command {
    pub fn handle_response(self, status: Status, headers: &[Header], body: &str) {
        match self {
            Command::Describe(describe) => describe.handle_response(status, headers, body),
        }
    }

    pub fn cancel(self, e: Error) {
        match self {
            Command::Describe(describe) => describe.cancel(e),
        }
    }

    pub fn url(&self) -> &url::Url {
        match self {
            Command::Describe(describe) => describe.url(),
        }
    }

    pub fn method(&self) -> Method {
        match self {
            Command::Describe(describe) => describe.method(),
        }
    }
}
