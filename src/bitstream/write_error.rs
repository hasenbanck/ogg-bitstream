//! Bitstream write errors.

use std::error::Error;

/// Errors that can occur when writing OGG bitstreams.
#[derive(Debug)]
pub enum BitstreamWriteError {
    /// A `std::io::Error`.
    IoError(std::io::Error),
    /// A `std::num::TryFromIntError`.
    TryFromIntError(std::num::TryFromIntError),
}

impl std::fmt::Display for BitstreamWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitstreamWriteError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            BitstreamWriteError::TryFromIntError(err) => {
                write!(f, "{:?}", err.source())
            }
        }
    }
}

impl From<std::io::Error> for BitstreamWriteError {
    fn from(err: std::io::Error) -> BitstreamWriteError {
        BitstreamWriteError::IoError(err)
    }
}

impl From<std::num::TryFromIntError> for BitstreamWriteError {
    fn from(err: std::num::TryFromIntError) -> BitstreamWriteError {
        BitstreamWriteError::TryFromIntError(err)
    }
}

impl std::error::Error for BitstreamWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            BitstreamWriteError::IoError(ref e) => Some(e),
            BitstreamWriteError::TryFromIntError(ref e) => Some(e),
        }
    }
}
