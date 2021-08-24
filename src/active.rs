use crate::prelude::*;
pub use crate::{Error, Result};
use tokio::io::AsyncReadExt;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;
//use tokio::select;

const MSGBOX: usize = 1024;

#[derive(Debug)]
enum SocketHandlerMsg {
    TrySend {
        msg: String,
    },
    Send {
        msg: String,
        reply_tx: oneshot::Sender<Result<()>>,
    },
    Close,
}

pub type IncomingMsg = String;

#[derive(Debug)]
pub struct Socket {
    send_handler: JoinHandle<()>,
    send_tx: mpsc::Sender<SocketHandlerMsg>,
    receive_handler: JoinHandle<()>,
    receive_tx: broadcast::Sender<IncomingMsg>,
    done_tx: Option<oneshot::Sender<()>>,
}

impl Socket {
    pub async fn connect(address: &SocketAddr) -> Result<Socket> {
        let stream = TcpStream::connect(address).await?;
        start_stream(stream)
    }

    pub fn try_send(&self) {
        self.send_tx
            .try_send(SocketHandlerMsg::TrySend {
                msg: String::from("msg"),
            })
            .unwrap();
    }

    pub async fn send(&self) -> Result<()> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.send_tx
            .send(SocketHandlerMsg::Send {
                msg: String::from("sendmsg"),
                reply_tx,
            })
            .await
            .unwrap();
        match reply_rx.await {
            Ok(reply) => reply,
            Err(err) => Err(Error::SocketSendHandlerDropped(err)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IncomingMsg> {
        return self.receive_tx.subscribe();
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        match self.done_tx.take() {
            Some(done_tx) => done_tx.send(()).unwrap(),
            None => (),
        }
        self.send_tx.try_send(SocketHandlerMsg::Close).unwrap(); //FIXME: handle error?
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

    let (send_tx, send_rx) = mpsc::channel(MSGBOX);
    let (receive_tx, _receive_rx) = broadcast::channel(MSGBOX);
    let (done_tx, done_rx) = oneshot::channel();

    let send_handler = tokio::spawn(async move {
        send_handler(w, send_rx).await;
    });

    let receive_tx_to_pass = receive_tx.clone();
    let receive_handler = tokio::spawn(async move {
        receive_handler(r, receive_tx_to_pass, done_rx).await;
    });

    Ok(Socket {
        send_handler,
        send_tx,
        receive_handler,
        receive_tx,
        done_tx: Some(done_tx),
    })
}

async fn send_handler(mut w: WriteHalf<TcpStream>, mut send_rx: mpsc::Receiver<SocketHandlerMsg>) {
    loop {
        match send_rx.recv().await {
            Some(SocketHandlerMsg::TrySend { msg }) => {
                //let string = String::from("msg");
                let buf = msg.as_bytes();
                if let Err(err) = w.write(&buf).await {
                    eprintln!("failed to write to socket: {:?}", err);
                }
            }
            Some(SocketHandlerMsg::Send { msg, reply_tx }) => {
                let buf = msg.as_bytes();
                let reply = match w.write(&buf).await {
                    Ok(_) => Ok(()),
                    Err(err) => Err(Error::IoError(err)),
                };
                let _ = reply_tx.send(reply);
            }

            Some(SocketHandlerMsg::Close) => {
                return ();
            }
            _unexpected => {
                eprintln!("unexpected stuff");
            }
        }
    }
}

async fn receive_handler(
    mut r: ReadHalf<TcpStream>,
    receive_tx: broadcast::Sender<IncomingMsg>,
    mut done_rx: oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = read_and_fw(&mut r, &receive_tx) => (),
            _ = (&mut done_rx) => return (),
        }
    }
}

async fn read_and_fw(r: &mut ReadHalf<TcpStream>, receive_tx: &broadcast::Sender<IncomingMsg>) {
    let mut buf = [0u8; 32];
    r.read(&mut buf).await.unwrap();
    //        println!("{:?}", std::str::from_utf8(&buf));
    receive_tx.send(String::from("msg")).unwrap(); // FIXME: handle error?
}
