use bincode;
use std::io;

/// The result of a serialization or deserialization operation.
pub type Result<T> = ::std::result::Result<T, Error>;

/// An error that can be produced during (de)serializing.
pub type Error = Box<ErrorKind>;


/// The kind of error that can be produced during a serialization or deserialization.
#[derive(Debug)]
pub enum ErrorKind {
    /// If the error stems from the reader/writer that is being used
    /// during (de)serialization, that error will be stored and returned here.
    Io(io::Error),
    Bincode(bincode::Error),
    /// A custom error message from Serde.
    Connection(),
    Custom(String),
}


impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        ErrorKind::Io(err).into()
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Error {
        ErrorKind::Bincode(err).into()
    }
}