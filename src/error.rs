use failure::Fail;
use std::io::Error as StdIoError;
use tokio_serial::Error as TokioSerialError;

/// Error type for dobot crate.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "the size of params can be up to 254 bytes")]
    ParamsTooLong,
    #[fail(display = "fail to deserialize message: {}", _0)]
    DeserializeError(String),
    #[fail(display = "io error: {:?}", _0)]
    IoError(StdIoError),
    #[fail(
        display = "checksum error: received {}, but it should be {}",
        received, expected
    )]
    IntegrityError { received: u8, expected: u8 },
    #[fail(display = "tokio-serial error: {}", _0)]
    AsyncIOError(TokioSerialError),
}

impl From<StdIoError> for Error {
    fn from(error: StdIoError) -> Self {
        Self::IoError(error)
    }
}
impl From<TokioSerialError> for Error {
    fn from(error: TokioSerialError) -> Self {
        Self::AsyncIOError(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
