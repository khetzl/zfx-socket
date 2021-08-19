pub mod active;
pub mod prelude;

#[derive(Debug)]
pub enum Error {
    HandlerReceiveError,
    HandlerError,
    IoError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

//impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}
