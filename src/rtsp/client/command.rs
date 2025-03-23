use crate::rtsp::protocol::*;
use crate::sdp;

use thiserror::Error;
use tokio::sync::oneshot;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ParseSdp(#[from] sdp::ParseError),
    #[error("Unexpected status code {0}")]
    UnexpectedStatus(Status),
    #[error("Cancelled")]
    Cancelled,
    #[error("Bad response")]
    BadResponse,
}

type PreparedBuilder<H> = RequestBuilder<H, VoidBody>;
type Result<T> = std::result::Result<T, Error>;
pub struct Describe {
    tx: oneshot::Sender<Result<sdp::Sdp>>,
}

impl Describe {
    pub fn write(&self, builder: PreparedBuilder<impl RequestWriter>, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(builder.method(Method::Describe).write(buf)?)
    }

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

    pub fn handle_error(self, e: Error) {
        let _ = self.tx.send(Err(e));
    }
}

impl Describe {
    pub fn new(tx: oneshot::Sender<Result<sdp::Sdp>>) -> Self {
        Self { tx }
    }
}

pub enum Command {
    Describe(Describe),
}

impl Command {
    pub fn write(&self, builder: PreparedBuilder<impl RequestWriter>, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Command::Describe(describe) => describe.write(builder, buf),
        }
    }

    pub fn handle_response(self, status: Status, headers: &[Header], body: &str) {
        match self {
            Command::Describe(describe) => describe.handle_response(status, headers, body),
        }
    }

    pub fn handle_error(self, e: Error) {
        match self {
            Command::Describe(describe) => describe.handle_error(e),
        }
    }
}
