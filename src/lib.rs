pub mod active;
pub mod prelude;

#[derive(Debug)]
pub enum Error {
    HandlerReceiveError,
    HandlerError,
    ConnectionError,
}

pub type Result<T> = std::result::Result<T, Error>;
