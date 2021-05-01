//! Bitstream write errors.

use std::error::Error;

/// Errors that can occur when writing OGG bitstreams.
#[derive(Debug)]
pub enum WriteError {
    /// A `std::io::Error`.
    IoError(std::io::Error),
    /// A `std::num::TryFromIntError`.
    TryFromIntError(std::num::TryFromIntError),
    /// Unknown bitstream serial number.
    UnknownBitstreamSerialNumber,
    /// Logical bitstream already initialized.
    BitstreamAlreadyInitialized,
    /// Initial packet too big.
    InitialPacketTooBig,
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            WriteError::TryFromIntError(err) => {
                write!(f, "{:?}", err.source())
            }
            WriteError::UnknownBitstreamSerialNumber => {
                write!(f, "unknown bitstream serial number")
            }
            WriteError::BitstreamAlreadyInitialized => {
                write!(f, "logical bitstream already initialized")
            }
            WriteError::InitialPacketTooBig => {
                write!(f, "initial packet too big. Max size: 65_025 byte")
            }
        }
    }
}

impl From<std::io::Error> for WriteError {
    fn from(err: std::io::Error) -> WriteError {
        WriteError::IoError(err)
    }
}

impl From<std::num::TryFromIntError> for WriteError {
    fn from(err: std::num::TryFromIntError) -> WriteError {
        WriteError::TryFromIntError(err)
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            WriteError::IoError(ref e) => Some(e),
            WriteError::TryFromIntError(ref e) => Some(e),
            _ => None,
        }
    }
}
