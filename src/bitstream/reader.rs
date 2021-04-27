use std::io::{Read, Seek};

use crate::{BitstreamReadError, Packet};

/// Returns the status of the read operation.
pub enum ReadStatus {
    /// Paket is fine.
    Ok,
    /// No new packet, since we reached the EOF.
    Eof,
    /// No new packet. Page was corrupted or packet is missing.
    Missing,
}

/// Generic OGG bitstream reader.
pub struct BitStreamReader {
    previous_packet_num: u64,
    /// caches the current page. As most
    page_buffer: Vec<u8>,
    /// Holds temporary data of a packet we try to assemble.
    segment_data: Vec<u8>,
}

impl BitStreamReader {
    /// Reads the next packet from the reader.
    ///
    /// Will gracefully handle recoverable errors like pages with wrong checksums,
    /// missing packets and out of sync events.
    ///
    /// Returns the status of the operation. When receiving `ReadStatus::MissingPacket` no data
    /// was written into the given frame.
    pub fn read_packet<R: Read>(
        _reader: &mut R,
        _packet: &mut Packet,
    ) -> Result<ReadStatus, BitstreamReadError> {
        todo!()
    }

    /// Seeks to the first packet after the given granular position.
    ///
    /// If the user is seeking outside of the stream, `read_packet()`
    /// will return `false` on the next call.
    pub fn seek<R: Read + Seek>(
        _reader: &mut R,
        _granular_position: u64,
    ) -> Result<(), BitstreamReadError> {
        todo!()
    }
}
