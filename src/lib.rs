#![warn(missing_docs)]
#![deny(unsafe_code)]
#![deny(clippy::as_conversions)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
//! Reads and writes OGG bitstreams.

use std::hash::{Hash, Hasher};
use std::ops::Range;

#[cfg(feature = "reader")]
pub use read_error::ReadError;
#[cfg(feature = "reader")]
pub use reader::{FileReader, Packet, ReadStatus, StreamReader};
#[cfg(feature = "writer")]
pub use write_error::WriteError;
#[cfg(feature = "writer")]
pub use writer::StreamWriter;

pub(crate) mod crc32;

#[cfg(feature = "reader")]
mod read_error;
#[cfg(feature = "reader")]
mod reader;

#[cfg(feature = "writer")]
mod write_error;
#[cfg(feature = "writer")]
mod writer;

pub(crate) const CONTINUATION_VALUE: u8 = 0x1;
pub(crate) const BOS_VALUE: u8 = 0x2;
pub(crate) const EOS_VALUE: u8 = 0x4;
pub(crate) const MAX_PAGE_HEADER_SIZE: usize = 27 + 255;
pub(crate) const MAX_PAGE_DATA_SIZE: usize = 65_025;
pub(crate) const MAX_PAGE_SIZE: usize = MAX_PAGE_HEADER_SIZE + MAX_PAGE_DATA_SIZE;
pub(crate) const PAGER_MARKER: [u8; 4] = [0x4F, 0x67, 0x67, 0x53];
pub(crate) const VERSION_INDEX: usize = 4;
pub(crate) const HEADER_TYPE_INDEX: usize = 5;
pub(crate) const SEGMENT_COUNT_INDEX: usize = 26;
pub(crate) const SEGMENT_TABLE_INDEX: usize = 27;
pub(crate) const HEADER_RANGE: Range<usize> = Range { start: 0, end: 27 };
pub(crate) const PAGER_MARKER_RANGE: Range<usize> = Range { start: 0, end: 4 };
pub(crate) const CONST_HEADER_DATA_RANGE: Range<usize> = Range { start: 4, end: 27 };
pub(crate) const GRANULE_POSITION_RANGE: Range<usize> = Range { start: 6, end: 14 };
pub(crate) const BITSTREAM_SERIAL_NUMBER_RANGE: Range<usize> = Range { start: 14, end: 18 };
pub(crate) const PAGE_SEQUENCE_NUMBER_RANGE: Range<usize> = Range { start: 18, end: 22 };
pub(crate) const CRC32_RANGE: Range<usize> = Range { start: 22, end: 26 };

#[inline]
pub(crate) fn parse_u32_le(source: &[u8]) -> u32 {
    let mut buffer = [0_u8; 4];
    buffer.copy_from_slice(&source[0..4]);
    u32::from_le_bytes(buffer)
}

#[inline]
pub(crate) fn parse_u64_le(source: &[u8]) -> u64 {
    let mut buffer = [0_u8; 8];
    buffer.copy_from_slice(&source[0..8]);
    u64::from_le_bytes(buffer)
}

/// Simple helper function to create a random bitstream serial number.
///
/// Uses the system time and default hasher to generate a random number.
/// Obvious to say this kind of randomness is low quality, but fine for
/// this specific use case.
#[cfg(feature = "writer")]
#[allow(clippy::as_conversions)]
pub fn generate_bitstream_serial_number() -> u32 {
    let now = std::time::SystemTime::now();
    let mut hasher = std::collections::hash_map::DefaultHasher::default();
    now.hash(&mut hasher);
    let hash = hasher.finish();
    (hash % u32::MAX as u64) as u32
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_generate_bitstream_serial_number() {
        let x1 = generate_bitstream_serial_number();
        let x2 = generate_bitstream_serial_number();
        assert_ne!(x1, x2);
    }
}
