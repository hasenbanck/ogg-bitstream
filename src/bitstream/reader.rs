use std::collections::VecDeque;
use std::error::Error;
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Range;

use crate::crc32::crc32_update;
use crate::{
    parse_u32_le, parse_u64_le, BitstreamReadError, BITSTREAM_SERIAL_NUMBER_RANGE,
    CONST_HEADER_DATA_RANGE, CRC32_RANGE, GRANULAR_POSITION_RANGE, HEADER_TYPE_INDEX,
    MAX_PAGE_SIZE, PAGER_MARKER, PAGE_SEQUENCE_NUMBER_RANGE, SEGMENT_COUNT_INDEX,
    SEGMENT_TABLE_INDEX, VERSION_INDEX,
};

/// A packet inside an OGG stream.
#[derive(Clone, Debug, Default)]
pub struct Packet {
    /// The data of the packet.
    data: Vec<u8>,
    /// Unique serial ID of the logical bitstream this packet belongs to.
    bitstream_serial_number: u32,
    /// The granule position of the last sample (`granule`) in the packet.
    granule_position: u64,
    /// Paket is a begin of stream marker.
    is_bos: bool,
    /// Paket is a end of stream marker.
    is_eos: bool,
}

impl Packet {
    /// The payload of the packet.
    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    /// Unique serial ID of the logical bitstream this packet belongs to.
    pub fn bitstream_serial_number(&self) -> u32 {
        self.bitstream_serial_number
    }

    /// The granule position of the last sample (`granule`) in the packet.
    pub fn granule_position(&self) -> u64 {
        self.granule_position
    }

    /// Paket has a begin of stream marker.
    pub fn is_bos(&self) -> bool {
        self.is_bos
    }

    /// Paket has a end of stream marker.
    pub fn is_eos(&self) -> bool {
        self.is_eos
    }
}

/// Returns the status of the read operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadStatus {
    /// Paket was written.
    Ok,
    /// No new packet, since we reached the EOF.
    Eof,
    /// No new packet. Page was corrupted or page didn't contain any packet.
    Missing,
}

#[derive(Clone, Debug)]
struct QueuedPacket {
    range: Range<usize>,
    is_complete: bool,
}

/// Generic OGG bitstream reader.
pub struct BitStreamReader {
    page_buffer: Box<[u8]>,
    queued_packets: VecDeque<QueuedPacket>,
    current_bitstream_serial_number: u32,
    current_page_sequence_number: u32,
    current_granule_position: u64,
    current_is_eos: bool,
}

impl Default for BitStreamReader {
    fn default() -> Self {
        Self {
            page_buffer: vec![0_u8; 65_307].into_boxed_slice(),
            queued_packets: VecDeque::with_capacity(32),
            current_bitstream_serial_number: 0,
            current_page_sequence_number: 0,
            current_granule_position: 0,
            current_is_eos: false,
        }
    }
}

impl BitStreamReader {
    /// Creates a new BitStreamReader.
    pub fn new() -> Self {
        Default::default()
    }

    /// Reads the next packet from the reader.
    ///
    /// Will gracefully handle recoverable errors like pages with wrong checksums,
    /// missing packets and out of sync events.
    ///
    /// Returns the status of the operation. When receiving `ReadStatus::MissingPacket` a page
    /// was corrupt / invalid and no data was written into the given frame.
    pub fn read_packet<R: Read>(
        &mut self,
        reader: &mut R,
        packet: &mut Packet,
    ) -> Result<ReadStatus, BitstreamReadError> {
        packet.data.clear();

        let is_last_packet = self.queued_packets.len() == 1;
        if let Some(queued_packet) = self.queued_packets.pop_front() {
            self.write_frame(packet, queued_packet.range)?;

            if is_last_packet && self.current_is_eos {
                packet.is_eos = true;
            }

            if queued_packet.is_complete {
                return Ok(ReadStatus::Ok);
            }
        }

        loop {
            if let Err(err) = self.sync_with_next_page(reader) {
                if let Some(err) = err.source() {
                    if err.downcast_ref::<std::io::Error>().is_some() {
                        return Ok(ReadStatus::Eof);
                    }
                }
                return Err(err);
            }

            let page_size = match self.read_page_data(reader) {
                Ok(page_size) => page_size,
                Err(err) => {
                    if let Some(err) = err.source() {
                        if err.downcast_ref::<std::io::Error>().is_some() {
                            return Ok(ReadStatus::Eof);
                        }
                    }
                    return Err(err);
                }
            };

            if !self.verify_crc32(page_size) {
                self.queued_packets.clear();
                packet.data.clear();

                return Ok(ReadStatus::Missing);
            }

            let version = self.page_buffer[VERSION_INDEX];

            let header_type = self.page_buffer[HEADER_TYPE_INDEX];
            let granule_position = parse_u64_le(&self.page_buffer[GRANULAR_POSITION_RANGE]);

            let bitstream_serial_number =
                parse_u32_le(&self.page_buffer[BITSTREAM_SERIAL_NUMBER_RANGE]);
            let page_sequence_number = parse_u32_le(&self.page_buffer[PAGE_SEQUENCE_NUMBER_RANGE]);

            let is_continuation = header_type & 0x1 == 1;
            let is_bos = (header_type & 0x2) >> 1 == 1;
            let is_eos = (header_type & 0x4) >> 2 == 1;

            if version != 0 {
                return Err(BitstreamReadError::UnhandledBitstreamVersion(version));
            }

            self.current_bitstream_serial_number = bitstream_serial_number;
            self.current_granule_position = granule_position;
            self.current_is_eos = is_eos;

            // Make sure we only append data to a previous, unfinished packet, if the page sequence is
            // sequential and the packet is from the same bitstream.
            if !packet.data.is_empty()
                && (self.current_bitstream_serial_number != bitstream_serial_number
                    || (self.current_page_sequence_number + 1) > page_sequence_number)
            {
                packet.data.clear();
            }

            return if let Some(queued_packet) = self.queued_packets.pop_front() {
                // Make sure we are actually appending to an unfinished packet.
                if is_continuation && !packet.data.is_empty() {
                    return Ok(ReadStatus::Missing);
                }

                self.write_frame(packet, queued_packet.range)?;

                if !queued_packet.is_complete {
                    continue;
                }

                if is_bos {
                    packet.is_bos = true;
                }

                Ok(ReadStatus::Ok)
            } else {
                Ok(ReadStatus::Missing)
            };
        }
    }

    fn write_frame(
        &mut self,
        packet: &mut Packet,
        data_range: Range<usize>,
    ) -> Result<(), BitstreamReadError> {
        packet.data.write_all(&self.page_buffer[data_range])?;
        packet.bitstream_serial_number = self.current_bitstream_serial_number;
        packet.granule_position = self.current_granule_position;
        packet.is_bos = false;
        packet.is_eos = false;

        Ok(())
    }

    fn sync_with_next_page<R: Read>(&self, reader: &mut R) -> Result<(), BitstreamReadError> {
        let mut marker_found = 0;
        for _ in 0..MAX_PAGE_SIZE {
            if marker_found == 4 {
                break;
            }
            let mut buffer = [0_u8; 1];
            reader.read_exact(&mut buffer)?;
            if buffer[0] == PAGER_MARKER[marker_found] {
                marker_found += 1;
            } else {
                marker_found = 0;
            }
        }

        Ok(())
    }

    fn verify_crc32(&mut self, page_size: usize) -> bool {
        let target_crc = parse_u32_le(&self.page_buffer[CRC32_RANGE]);
        self.page_buffer[CRC32_RANGE]
            .iter_mut()
            .for_each(|x| *x = 0);

        let crc32 = crc32_update(0, &self.page_buffer[..page_size]);

        target_crc == crc32
    }

    fn read_page_data<R: Read>(&mut self, reader: &mut R) -> Result<usize, BitstreamReadError> {
        PAGER_MARKER
            .iter()
            .enumerate()
            .for_each(|(i, x)| self.page_buffer[i] = *x);
        reader.read_exact(&mut self.page_buffer[CONST_HEADER_DATA_RANGE])?;

        // Read the packet offsets from the segment table.
        let table_size = usize::from(self.page_buffer[SEGMENT_COUNT_INDEX]);
        let table_start = SEGMENT_TABLE_INDEX;
        let table_end = SEGMENT_TABLE_INDEX + table_size;
        reader.read_exact(&mut self.page_buffer[table_start..table_end])?;

        let mut segment_size = 0;
        let mut read_size = 0;
        for lace in self.page_buffer[table_start..table_end].iter() {
            let bytes = usize::from(*lace);
            segment_size += bytes;

            match bytes {
                255 => continue,
                _ => {
                    let queued_packet = QueuedPacket {
                        range: table_end + read_size..table_end + read_size + segment_size,
                        is_complete: true,
                    };
                    read_size += segment_size;
                    segment_size = 0;

                    self.queued_packets.push_back(queued_packet);
                }
            }
        }

        // Handle unfinished packets. They can occur when a packet is bigger than an OGG page.
        if segment_size != 0 {
            // TODO create a file later that tests this case.
            let queued_packet = QueuedPacket {
                range: table_end + read_size..table_end + read_size + segment_size,
                is_complete: false,
            };
            read_size += segment_size;

            self.queued_packets.push_back(queued_packet);
        }

        // Copy the payload data.
        let page_end = table_start + table_size + read_size;
        reader.read_exact(&mut self.page_buffer[table_end..page_end])?;

        Ok(page_end)
    }

    /// Seeks to the first packet after the given granule position.
    ///
    /// If the user is seeking outside of the stream, `read_packet()`
    /// will return `false` on the next call.
    ///
    /// Be sure to use a `BuffRead` buffer for IO devices for better
    /// performance.
    pub fn seek<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        target_granule_position: u64,
    ) -> Result<(), BitstreamReadError> {
        // The seek is currently implemented as a simple binary search.
        // There is a lot of room for improvement!

        if target_granule_position == u64::MAX {
            reader.seek(SeekFrom::End(0))?;
            return Ok(());
        }

        if target_granule_position == 0 {
            reader.seek(SeekFrom::Start(0))?;
            return Ok(());
        }

        let mut buffer = [0_u8; 10];
        let mut granule_position: u64;

        let mut left = 0;
        let mut right = reader.seek(SeekFrom::End(0))?;

        if target_granule_position >= right {
            return Ok(());
        }

        let mut mid: u64 = 0;
        while left < right {
            mid = (left + right) / 2;

            reader.seek(SeekFrom::Start(mid))?;

            // Exit when we are close enough.
            if (right - left) < 100 {
                return Ok(());
            }

            loop {
                self.sync_with_next_page(reader)?;
                reader.read_exact(&mut buffer)?;
                granule_position = parse_u64_le(&buffer[2..]);

                if granule_position != u64::MAX {
                    break;
                }
            }

            match granule_position {
                g if g < target_granule_position => left = mid + 1,
                g if g > target_granule_position => right = mid - 1,
                _ => break,
            }
        }
        reader.seek(SeekFrom::Start(mid))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    #![allow(clippy::unwrap_used)]

    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_sync() {
        let d: Vec<u8> = vec![
            0x4F, 0x67, 0x67, 0x53, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x4A, 0xC9, 0x09, 0xB6, 0x00, 0x00, 0x00, 0x00, 0xF9, 0x20, 0x89, 0xF8, 0x01, 0x13,
            0x4F, 0x70, 0x75, 0x73, 0x48, 0x65, 0x61, 0x64, 0x01, 0x02, 0x38, 0x01, 0x80, 0xBB,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut c = Cursor::new(d);

        let mut br = BitStreamReader::new();
        let mut packet = Packet::default();
        let res = br.read_packet(&mut c, &mut packet).unwrap();
        assert_eq!(res, ReadStatus::Ok)
    }

    #[test]
    fn test_resync() {
        let d: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x4F, 0x67, 0x67, 0x53, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x4A, 0xC9, 0x09, 0xB6, 0x00, 0x00, 0x00, 0x00, 0xF9,
            0x20, 0x89, 0xF8, 0x01, 0x13, 0x4F, 0x70, 0x75, 0x73, 0x48, 0x65, 0x61, 0x64, 0x01,
            0x02, 0x38, 0x01, 0x80, 0xBB, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut c = Cursor::new(d);

        let mut br = BitStreamReader::new();
        let mut packet = Packet::default();
        let res = br.read_packet(&mut c, &mut packet).unwrap();
        assert_eq!(res, ReadStatus::Ok)
    }
}
