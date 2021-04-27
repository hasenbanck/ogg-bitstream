//! Bitsream read errors.

use std::error::Error;

/// Errors that can occur when reading OGG bitstreams.
#[derive(Debug)]
pub enum BitstreamReadError {
    /// A `std::io::Error`.
    IoError(std::io::Error),
    /// A `std::num::TryFromIntError`.
    TryFromIntError(std::num::TryFromIntError),
}

impl std::fmt::Display for BitstreamReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitstreamReadError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            BitstreamReadError::TryFromIntError(err) => {
                write!(f, "{:?}", err.source())
            }
        }
    }
}

impl From<std::io::Error> for BitstreamReadError {
    fn from(err: std::io::Error) -> BitstreamReadError {
        BitstreamReadError::IoError(err)
    }
}

impl From<std::num::TryFromIntError> for BitstreamReadError {
    fn from(err: std::num::TryFromIntError) -> BitstreamReadError {
        BitstreamReadError::TryFromIntError(err)
    }
}

impl std::error::Error for BitstreamReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            BitstreamReadError::IoError(ref e) => Some(e),
            BitstreamReadError::TryFromIntError(ref e) => Some(e),
        }
    }
}
