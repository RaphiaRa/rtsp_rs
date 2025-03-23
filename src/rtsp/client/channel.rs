use super::*;
use crate::rtp;
use crate::rtsp::*;
use rustls_pki_types::InvalidDnsNameError;
use std::collections::HashMap;
use std::vec;
use thiserror::Error;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot};
use url::Url;

#[derive(Debug, Error)]
enum ClientError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    InvalidDnsName(#[from] InvalidDnsNameError),
    #[error(transparent)]
    ParseResponse(#[from] ParseError),
    #[error("Unexpected status code {0}")]
    UnexpectedStatus(Status),
    #[error(transparent)]
    Encoding(#[from] std::str::Utf8Error),
    #[error("Response header too long")]
    HeaderTooLong,
    #[error("Request too long")]
    RequestTooLong,
    #[error("Out of buffer space")]
    BufferError(#[from] BufferError),
    #[error("Incomplete response")]
    IncompleteResponse,
    #[error("Invalid CSeq")]
    InvalidCSeq,
}

type Result<T> = std::result::Result<T, ClientError>;

type CSeq = u32;

pub struct Channel<Stream> {
    stream: Stream,
    url: Url,
    cseq: CSeq,
    buffer_rx: Buffer,
    buffer_tx: Buffer,
    cmd_rx: mpsc::Receiver<Command>,
    cmd_pending: HashMap<CSeq, Command>,

    // For sending processed packets to the client
    packet_tx: mpsc::Sender<rtp::Packet>,
    shutdown: bool,
}

impl<Stream: AsyncReadExt + AsyncWriteExt + Send + Unpin + 'static> Channel<Stream> {
    fn new(stream: Stream, url: Url, cmd_rx: mpsc::Receiver<Command>, packet_tx: mpsc::Sender<rtp::Packet>) -> Self {
        Self {
            stream,
            url,
            cseq: 1,
            buffer_rx: Buffer::new(512 * 1024),
            buffer_tx: Buffer::new(512 * 1024),
            cmd_rx,
            cmd_pending: HashMap::new(),
            packet_tx,
            shutdown: false,
        }
    }

    fn read_rtsp_packet(&mut self) -> Result<usize> {
        let read_buf = self.buffer_rx.get_read_slice();
        let mut cseq: Option<CSeq> = None;
        let mut status: Option<Status> = None;
        let mut body: Option<&str> = None;
        let mut headers: Vec<Header> = Vec::new();
        let mut parser = ResponseParser::new();
        while let Some(item) = parser.parse_next(read_buf)? {
            match item {
                ParseItem::Header(h) => {
                    if h.name.eq_ignore_ascii_case("cseq") {
                        cseq = Some(h.value.parse().map_err(|_| ClientError::InvalidCSeq)?);
                    } else {
                        headers.push(Header::new(h.name, h.value));
                    }
                }
                ParseItem::Status(s) => {
                    status = Some(s);
                }
                ParseItem::Body(b) => {
                    body = Some(b);
                }
                _ => {}
            }
        }
        if !parser.is_done() {
            let bytes = parser.missing_bytes().ok_or(if read_buf.len() > 1024 {
                ClientError::HeaderTooLong
            } else {
                ClientError::IncompleteResponse
            })?;
            if bytes > 32 * 1024 {
                return Err(ClientError::RequestTooLong);
            } else {
                return Err(ClientError::IncompleteResponse);
            }
        }
        let cseq = cseq.ok_or(ClientError::InvalidCSeq)?;
        println!("cseq: {}", cseq);
        for key in self.cmd_pending.keys() {
            println!("key: {}", key);
        }
        let cmd = self.cmd_pending.remove(&cseq).ok_or(ClientError::InvalidCSeq)?;
        if let Some(status) = status {
            cmd.handle_response(status, &headers, body.ok_or(ClientError::IncompleteResponse)?);
        } else {
            cmd.handle_error(CommandError::BadResponse);
        }
        println!("Parsed {} bytes", parser.parsed_bytes());
        Ok(parser.parsed_bytes())
    }

    fn read_rtp_or_rtcp_packet(&mut self) -> Result<usize> {
        Ok(0)
    }

    fn read_packet(&mut self) -> Result<usize> {
        let read_buf = self.buffer_rx.get_read_slice();
        if read_buf.is_empty() {
            return Ok(0);
        }
        // check if we have a rtp/rtcp packet i.e the first byte is '$'
        if read_buf[0] == b'$' {
            self.read_rtp_or_rtcp_packet()
        } else {
            self.read_rtsp_packet()
        }
    }

    fn handle_data(&mut self) {
        loop {
            match self.read_packet() {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    self.buffer_rx.notify_read(n);
                }
                Err(e) => match e {
                    ClientError::IncompleteResponse => {
                        break; // Simply retry later
                    }
                    _ => {
                        println!("Error reading packet: {}, shutdown", e);
                        log::error!("Error reading packet: {}, shutdown", e);
                        self.shutdown();
                        break;
                    }
                },
            }
        }
    }

    fn shutdown(&mut self) {
        self.shutdown = true;
        for (_, cmd) in self.cmd_pending.drain() {
            cmd.handle_error(CommandError::Cancelled);
        }
    }

    async fn send_outstanding_data(&mut self) -> Result<()> {
        let write_buf = self.buffer_tx.get_read_slice();
        if !write_buf.is_empty() {
            let result = self.stream.write_all(write_buf).await;
            match result {
                Ok(_) => {
                    let n = write_buf.len();
                    self.buffer_tx.notify_read(n);
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    async fn poll_until_shutdown(&mut self) -> Result<()> {
        while !self.shutdown {
            // write all the data we got in the buffer
            self.send_outstanding_data().await?;
            let mut read_buf = self.buffer_rx.get_write_slice(4096).unwrap();
            tokio::select! {
                result = self.stream.read(&mut read_buf) => {
                    match result {
                        Ok(n) => {
                            if n == 0 {
                                log::info!("Stream closed");
                                break;
                            }
                            self.buffer_rx.notify_write(n);
                            self.handle_data();
                        }
                        Err(e) => {
                            log::error!("Error reading from stream: {}", e);
                            break;
                        }
                    }
                },
                Some(cmd) = self.cmd_rx.recv() => {
                    self.handle_command(cmd);
                }
            }
        }
        Ok(())
    }

    fn next_cseq(&mut self) -> CSeq {
        let cseq = self.cseq;
        self.cseq += 1;
        cseq
    }

    fn handle_command(&mut self, cmd: Command) {
        let cseq = self.next_cseq();
        let mut write_buf = self.buffer_tx.get_write_slice(4096).unwrap();
        let builder = RequestBuilder::new(self.url.clone())
            .header("CSeq", cseq)
            .header("User-Agent", "rs-streamer");
        let n = cmd.write(builder, &mut write_buf).unwrap();
        self.buffer_tx.notify_write(n);
        self.cmd_pending.insert(cseq, cmd);
    }

    async fn run(mut self) {
        let result = self.poll_until_shutdown().await;
        if let Err(e) = result {
            log::error!("Stream shutdown with error: {}", e);
        }
    }

    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn(self.run())
    }
}



#[cfg(test)]
use std::io::Write;
#[tokio::test]
async fn test_channel() {
    use command::Describe;

    let (cmd_tx, cmd_rx) = mpsc::channel(8);
    let (packet_tx, packet_rx) = mpsc::channel(8);
    let (cstream, sstream) = tokio::io::duplex(4096);
    tokio::spawn(async move {
        let mut sstream = sstream;
        let mut read_buf = vec![0u8; 4096];
        let n = sstream.read(&mut read_buf).await.unwrap();
        assert_eq!(&read_buf[..n], b"DESCRIBE rtsp://test.com RTSP/1.0\r\nCSeq: 1\r\nUser-Agent: rs-streamer\r\n\r\n");

        let mut write_buf = Vec::<u8>::new();
        write!(write_buf, "RTSP/1.0 200 OK\r\nCSeq: 1\r\nContent-Length: 4\r\n\r\ntest").unwrap();
        sstream.write_all(&write_buf).await.unwrap();
    });
    let url = Url::parse("rtsp://test.com").unwrap();
    let channel = Channel::new(cstream, url, cmd_rx, packet_tx);
    let handle = channel.start();
    let (tx, rx) = oneshot::channel();
    let cmd = Command::Describe(Describe::new(tx));
    cmd_tx.send(cmd).await.unwrap();
    let response = rx.await.unwrap().unwrap();
    handle.await.unwrap();
}
