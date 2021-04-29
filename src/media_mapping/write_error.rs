//! Media writer errors.

use std::error::Error;

/// Errors thrown by a media writer.
#[derive(Debug)]
pub enum MediaWriteError {
    /// A `std::io::Error`.
    IoError(std::io::Error),
    /// A `std::num::TryFromIntError`.
    TryFromIntError(std::num::TryFromIntError),
}

impl std::fmt::Display for MediaWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaWriteError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            MediaWriteError::TryFromIntError(err) => {
                write!(f, "{:?}", err.source())
            }
        }
    }
}

impl From<std::io::Error> for MediaWriteError {
    fn from(err: std::io::Error) -> MediaWriteError {
        MediaWriteError::IoError(err)
    }
}

impl From<std::num::TryFromIntError> for MediaWriteError {
    fn from(err: std::num::TryFromIntError) -> MediaWriteError {
        MediaWriteError::TryFromIntError(err)
    }
}

impl std::error::Error for MediaWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            MediaWriteError::IoError(ref e) => Some(e),
            MediaWriteError::TryFromIntError(ref e) => Some(e),
        }
    }
}
