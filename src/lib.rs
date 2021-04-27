#![warn(missing_docs)]
#![deny(unsafe_code)]
#![deny(unused_results)]
#![deny(clippy::as_conversions)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
//! Reads and writes OGG container files / streams.

pub use bitstream::*;
pub use media_mapping::*;

mod bitstream;
mod media_mapping;
