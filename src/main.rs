mod rtp;
mod rtsp;
mod rtcp;
mod sdp;
mod http;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
//mod types;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let (cmd_tx, cmd_rx) = mpsc::channel::<rtsp::client::Command>(8);
    let (packet_tx, packet_rx) = mpsc::channel::<rtp::Packet>(8);
    // create a socket connected to 192.168.2.31
    let host = "192.168.178.31:554";
    let socket = tokio::net::TcpStream::connect(host).await.unwrap();
    let channel = rtsp::client::Channel::new(socket, cmd_rx, packet_tx).user("admin").pass("Instar1!");
    let handle = channel.start();
    let (tx, rx) = oneshot::channel::<rtsp::client::CommandResult<sdp::Sdp>>();
    let describe = rtsp::client::Describe::new(url::Url::parse(&format!("rtsp://{}", host)).unwrap(), tx);
    let cmd = rtsp::client::Command::Describe(describe);
    cmd_tx.send(cmd).await.unwrap();
    let result = rx.await.unwrap();
    match result {
        Ok(sdp) => {
            println!("SDP: {:?}", sdp);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    handle.await.unwrap();
}
