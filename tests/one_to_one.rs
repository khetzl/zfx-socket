#[cfg(test)]
use std::net::SocketAddr;
use zfx_socket::active::Socket;
use zfx_socket::Error;

#[tokio::test]
async fn failure_to_connect_due_to_no_listener() {
    let target_address = "127.0.0.1:9090";
    let addr = target_address.parse::<SocketAddr>().unwrap();
    let s = Socket::connect(addr).await;
    match s {
        Err(Error::IoError(std::io::Error { .. })) => (),
        Err(err) => panic!("Unexpected error: {:?}", err),
        Ok(socket) => {
            socket.close();
            panic!("Socket shouldn't be ");
        }
    }
}
