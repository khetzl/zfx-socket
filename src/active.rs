use crate::prelude::*;
pub use crate::{Error, Result};

enum SocketHandlerMsg {
    Send { msg: String },
    Close,
}

#[derive(Debug)]
pub struct Socket {
    handler: JoinHandle<()>,
    send_tx: mpsc::Sender<SocketHandlerMsg>,
}

impl Socket {
    pub async fn connect(address: SocketAddr) -> Result<Socket> {
        let (send_tx, send_rx) = mpsc::channel(32); //FIXME: channel size?
        let (ready_tx, ready_rx) = oneshot::channel();
        //        let (receive_tx, receive_rx) = broad
        let handler = tokio::spawn(async move {
            client_handler(address, send_rx, ready_tx).await;
        });

        match ready_rx.await {
            Err(err) => Err(Error::HandlerReceiveError),
            Ok(Err(err)) => Err(Error::ConnectionError),
            Ok(Ok(())) => Ok(Socket { handler, send_tx }),
        }
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

async fn client_handler(
    address: SocketAddr,
    mut send_rx: mpsc::Receiver<SocketHandlerMsg>,
    ready_tx: oneshot::Sender<Result<()>>,
) {
    let mut stream: TcpStream;
    let connect_result = TcpStream::connect(&address).await;
    match connect_result {
        Err(err) => {
            ready_tx
                .send(Err(Error::ConnectionError))
                .expect("ERROR: while sending conenct error");
            return ();
        }
        Ok(accepted_connection) => {
            stream = accepted_connection;
            ready_tx
                .send(Ok(()))
                .expect("ERROR: while sending ready sign");
        }
    }
    loop {
        match send_rx.recv().await {
            Some(SocketHandlerMsg::Send { msg }) => {
                if let Err(err) = stream.write(msg.as_bytes()).await {
                    eprintln!("failed to write to socket");
                }
            }
            Some(SocketHandlerMsg::Close) => {
                println!("close socket")
            }
            _ => eprintln!("unexpected stuff"),
        }
    }
}

async fn listen_loop() {}
