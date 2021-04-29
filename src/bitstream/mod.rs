use std::ops::Range;

#[cfg(feature = "decoder")]
pub use read_error::BitstreamReadError;
#[cfg(feature = "decoder")]
pub use reader::{BitStreamFileReader, BitStreamStreamReader, Packet, ReadStatus};
#[cfg(feature = "encoder")]
pub use write_error::BitstreamWriteError;
#[cfg(feature = "encoder")]
pub use writer::{BitStreamWriter, PacketType};

pub(crate) mod crc32;

#[cfg(feature = "decoder")]
mod read_error;
#[cfg(feature = "decoder")]
mod reader;

#[cfg(feature = "encoder")]
mod write_error;
#[cfg(feature = "encoder")]
mod writer;

pub(crate) const MAX_PAGE_SIZE: usize = 65_307;
pub(crate) const PAGER_MARKER: [u8; 4] = [0x4F, 0x67, 0x67, 0x53];
pub(crate) const VERSION_INDEX: usize = 4;
pub(crate) const HEADER_TYPE_INDEX: usize = 5;
pub(crate) const SEGMENT_COUNT_INDEX: usize = 26;
pub(crate) const SEGMENT_TABLE_INDEX: usize = 27;
pub(crate) const HEADER_RANGE: Range<usize> = Range { start: 0, end: 27 };
pub(crate) const CONST_HEADER_DATA_RANGE: Range<usize> = Range { start: 4, end: 27 };
pub(crate) const GRANULAR_POSITION_RANGE: Range<usize> = Range { start: 6, end: 14 };
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
