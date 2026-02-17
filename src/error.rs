use embedded_bacnet::simple::BacnetError;

use crate::io::TokioUdpIo;

/// Error type for async-bacnet operations.
#[derive(Debug)]
pub enum Error {
    /// I/O error from socket operations.
    Io(std::io::Error),
    /// BACnet protocol error from embedded-bacnet.
    Bacnet(BacnetError<TokioUdpIo>),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<BacnetError<TokioUdpIo>> for Error {
    fn from(value: BacnetError<TokioUdpIo>) -> Self {
        Error::Bacnet(value)
    }
}
