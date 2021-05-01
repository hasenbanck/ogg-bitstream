//! Bitstream read errors.

use std::error::Error;

/// Errors that can occur when reading OGG bitstreams.
#[derive(Debug)]
pub enum ReadError {
    /// A `std::io::Error`.
    IoError(std::io::Error),
    /// A `std::num::TryFromIntError`.
    TryFromIntError(std::num::TryFromIntError),
    /// Reader only supports bitstreams of version `0`.
    UnhandledBitstreamVersion(u8),
    /// Unable to sync.
    UnableToSync,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            ReadError::TryFromIntError(err) => {
                write!(f, "{:?}", err.source())
            }
            ReadError::UnhandledBitstreamVersion(version) => {
                write!(
                    f,
                    "reader only supports bitstreams of version `0`. Found version: {}",
                    version
                )
            }
            ReadError::UnableToSync => {
                write!(f, "can't sync the next page")
            }
        }
    }
}

impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> ReadError {
        ReadError::IoError(err)
    }
}

impl From<std::num::TryFromIntError> for ReadError {
    fn from(err: std::num::TryFromIntError) -> ReadError {
        ReadError::TryFromIntError(err)
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ReadError::IoError(ref e) => Some(e),
            ReadError::TryFromIntError(ref e) => Some(e),
            _ => None,
        }
    }
}
