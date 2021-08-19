use crate::prelude::*;
pub use crate::{Error, Result};
use tokio::io::AsyncReadExt;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;

enum SocketHandlerMsg {
    Send { msg: String },
    Close,
}

pub type IncomingMsg = String;

#[derive(Debug)]
pub struct Socket {
    send_handler: JoinHandle<()>,
    send_tx: mpsc::Sender<SocketHandlerMsg>,
    receive_handler: JoinHandle<()>,
    receive_rx: broadcast::Receiver<IncomingMsg>,
}

impl Socket {
    pub async fn connect(address: SocketAddr) -> Result<Socket> {
        let mut stream = TcpStream::connect(&address).await?;
        start_stream(stream)
        /*
                let (r, w) = stream.split();

                let (send_tx, send_rx) = mpsc::channel(32); //FIXME: channel size?
                let (receive_tx, receive_rx) = broadcast::channel(32); //FIXME: channel size?

                let send_handler = tokio::spawn(async move {
                    send_handler(send_rx).await;
                });

                let receive_handler = tokio::spawn(async move {
                    receive_handler(receive_tx).await;
                });

                Ok(Socket {
                    send_handler,
                    send_tx,
                    receive_handler,
                    receive_rx,
                })
        */
    }

    pub fn try_send(&self) {
        self.send_tx.try_send(SocketHandlerMsg::Send {
            msg: String::from("msg"),
        });
    }

    pub fn close(&self) {
        self.send_tx.try_send(SocketHandlerMsg::Close);
    }
}

pub struct Listener {
    listener: TcpListener,
}

impl Listener {
    pub async fn bind(address: SocketAddr) -> Result<Listener> {
        let listener = TcpListener::bind(&address).await?;
        Ok(Listener { listener })
    }

    pub async fn accept(&self) -> Result<Socket> {
        let (stream, _) = self.listener.accept().await?;
        start_stream(stream)
    }
}

fn start_stream(mut stream: TcpStream) -> Result<Socket> {
    let (r, w) = tokio::io::split(stream);
    // let (r, w) = stream.split();

    let (send_tx, send_rx) = mpsc::channel(32); //FIXME: channel size?
    let (receive_tx, receive_rx) = broadcast::channel(32); //FIXME: channel size?

    let send_handler = tokio::spawn(async move {
        send_handler(w, send_rx).await;
    });

    let receive_handler = tokio::spawn(async move {
        receive_handler(r, receive_tx).await;
    });

    Ok(Socket {
        send_handler,
        send_tx,
        receive_handler,
        receive_rx,
    })
}

async fn send_handler(mut w: WriteHalf<TcpStream>, mut send_rx: mpsc::Receiver<SocketHandlerMsg>) {
    loop {
        match send_rx.recv().await {
            Some(SocketHandlerMsg::Send { msg }) => {
                // if let Err(err) = stream.write(msg.as_bytes()).await {
                eprintln!("failed to write to socket");
                // }
            }
            Some(SocketHandlerMsg::Close) => {
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
