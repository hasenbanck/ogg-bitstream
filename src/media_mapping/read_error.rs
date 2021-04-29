//! Media reader errors.

use std::error::Error;

/// Errors thrown by a media reader.
#[derive(Debug)]
pub enum MediaReadError {
    /// A `std::io::Error`.
    IoError(std::io::Error),
    /// A `std::num::TryFromIntError`.
    TryFromIntError(std::num::TryFromIntError),
}

impl std::fmt::Display for MediaReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaReadError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            MediaReadError::TryFromIntError(err) => {
                write!(f, "{:?}", err.source())
            }
        }
    }
}

impl From<std::io::Error> for MediaReadError {
    fn from(err: std::io::Error) -> MediaReadError {
        MediaReadError::IoError(err)
    }
}

impl From<std::num::TryFromIntError> for MediaReadError {
    fn from(err: std::num::TryFromIntError) -> MediaReadError {
        MediaReadError::TryFromIntError(err)
    }
}

impl std::error::Error for MediaReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            MediaReadError::IoError(ref e) => Some(e),
            MediaReadError::TryFromIntError(ref e) => Some(e),
        }
    }
}
