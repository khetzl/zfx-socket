use crate::prelude::*;
pub use crate::{Error, Result};
use tokio::io::AsyncReadExt;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;

enum SocketHandlerMsg {
    Send { msg: String }, // Send message
    Close,                // Close on request
    RemoteClosed,         // Remote endpoint closed the connection
}

pub type IncomingMsg = String;

#[derive(Debug)]
pub struct Socket {
    send_handler: JoinHandle<()>,
    send_tx: mpsc::Sender<SocketHandlerMsg>,
    receive_handler: JoinHandle<()>,
    receive_tx: broadcast::Sender<IncomingMsg>,
    //pub receive_rx: broadcast::Receiver<IncomingMsg>,
}

impl Socket {
    pub async fn connect(address: &SocketAddr) -> Result<Socket> {
        let stream = TcpStream::connect(address).await?;
        start_stream(stream)
    }

    pub fn try_send(&self) {
        self.send_tx.try_send(SocketHandlerMsg::Send {
            msg: String::from("msg"),
        });
    }

    pub fn close(&self) {
        self.send_tx.try_send(SocketHandlerMsg::Close);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IncomingMsg> {
        return self.receive_tx.subscribe();
    }
}

pub struct Listener {
    listener: TcpListener,
}

impl Listener {
    pub async fn bind(address: &SocketAddr) -> Result<Listener> {
        let listener = TcpListener::bind(address).await?;
        Ok(Listener { listener })
    }

    pub async fn accept(&self) -> Result<Socket> {
        let (stream, _) = self.listener.accept().await?;
        start_stream(stream)
    }
}

fn start_stream(stream: TcpStream) -> Result<Socket> {
    let (r, w) = tokio::io::split(stream);

    let (send_tx, send_rx) = mpsc::channel(32); //FIXME: channel size?
    let (receive_tx, receive_rx) = broadcast::channel(32); //FIXME: channel size?

    let send_handler = tokio::spawn(async move {
        send_handler(w, send_rx).await;
    });

    let receive_tx_to_pass = receive_tx.clone();
    let receive_handler = tokio::spawn(async move {
        receive_handler(r, receive_tx_to_pass).await;
    });

    Ok(Socket {
        send_handler,
        send_tx,
        receive_handler,
        receive_tx,
        //receive_rx,
    })
}

async fn send_handler(mut w: WriteHalf<TcpStream>, mut send_rx: mpsc::Receiver<SocketHandlerMsg>) {
    loop {
        match send_rx.recv().await {
            Some(SocketHandlerMsg::Send { msg }) => {
                let string = String::from("msg");
                let buf = string.as_bytes();
                w.write(&buf).await.unwrap();
                // if let Err(err) = stream.write(msg.as_bytes()).await {
                eprintln!("failed to write to socket");
                // }
            }
            Some(SocketHandlerMsg::Close) => {
                // TODO: should shut down receive_handler as well.
                return ();
            }
            _ => eprintln!("unexpected stuff"),
        }
    }
}

async fn receive_handler(mut r: ReadHalf<TcpStream>, receive_tx: broadcast::Sender<IncomingMsg>) {
    loop {
        let mut buf = [0u8; 32];
        r.read(&mut buf).await.unwrap();
        println!("{:?}", std::str::from_utf8(&buf));
        receive_tx.send(String::from("msg"));
    }
}
