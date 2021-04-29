use std::collections::VecDeque;
use std::convert::TryFrom;
use std::error::Error;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::ops::Range;

use crate::crc32::crc32_update;
use crate::{
    parse_u32_le, parse_u64_le, BitstreamReadError, BITSTREAM_SERIAL_NUMBER_RANGE,
    CONST_HEADER_DATA_RANGE, CRC32_RANGE, GRANULAR_POSITION_RANGE, HEADER_RANGE, HEADER_TYPE_INDEX,
    MAX_PAGE_SIZE, PAGER_MARKER, PAGE_SEQUENCE_NUMBER_RANGE, SEGMENT_COUNT_INDEX,
    SEGMENT_TABLE_INDEX, VERSION_INDEX,
};

macro_rules! handle_eof {
    ($err:ident, $action:expr) => {
        if let Some(err) = $err.source() {
            if err.downcast_ref::<std::io::Error>().is_some() {
                $action;
            }
        }
        return Err($err);
    };
}

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

/// Generic OGG bitstream file reader.
#[derive(Clone, Debug)]
pub struct BitStreamFileReader<R: Read + Seek> {
    inner: BitStreamReader,
    reader: R,
}

impl<R: Read + Seek> BitStreamFileReader<R> {
    /// Creates a new `BitStreamFileReader`.
    pub fn new(reader: R) -> Self {
        Self {
            inner: Default::default(),
            reader,
        }
    }

    /// Consumes the `BitStreamFileReader` and returns the reader.
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Reads the next packet from the reader.
    ///
    /// Will gracefully handle recoverable errors like pages with wrong checksums,
    /// missing packets and out of sync events.
    ///
    /// Returns the status of the operation. When receiving `ReadStatus::MissingPacket` a page
    /// was corrupt / invalid and no data was written into the given frame.
    pub fn read_packet(&mut self, packet: &mut Packet) -> Result<ReadStatus, BitstreamReadError> {
        self.inner.read_packet(&mut self.reader, packet)
    }

    /// Seeks to the first page that has an granule position greater or equal
    /// to th given one for the given logical bitstream.
    ///
    /// If the user is seeking outside of the stream, `read_packet()`
    /// will return the packets of the last page.
    pub fn seek(
        &mut self,
        bitstream_serial_number: u32,
        target_granule_position: u64,
    ) -> Result<(), BitstreamReadError> {
        self.inner.seek(
            &mut self.reader,
            bitstream_serial_number,
            target_granule_position,
        )
    }
}

/// Generic OGG bitstream stream reader.
#[derive(Clone, Debug)]
pub struct BitStreamStreamReader<R: Read> {
    inner: BitStreamReader,
    reader: R,
}

impl<R: Read> BitStreamStreamReader<R> {
    /// Creates a new `BitStreamStreamReader`.
    pub fn new(reader: R) -> Self {
        Self {
            inner: Default::default(),
            reader,
        }
    }

    /// Consumes the `BitStreamStreamReader` and returns the reader.
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Reads the next packet from the reader.
    ///
    /// Will gracefully handle recoverable errors like pages with wrong checksums,
    /// missing packets and out of sync events.
    ///
    /// Returns the status of the operation. When receiving `ReadStatus::MissingPacket` a page
    /// was corrupt / invalid and no data was written into the given frame.
    pub fn read_packet(&mut self, packet: &mut Packet) -> Result<ReadStatus, BitstreamReadError> {
        self.inner.read_packet(&mut self.reader, packet)
    }
}

#[derive(Clone, Debug)]
struct BitStreamReader {
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
    fn read_packet<R: Read>(
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
                handle_eof!(err, return Ok(ReadStatus::Eof));
            }

            let page_size = match self.read_page_data(reader) {
                Ok(page_size) => page_size,
                Err(err) => {
                    handle_eof!(err, return Ok(ReadStatus::Eof));
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

            // Make sure we only append data to a previous, unfinished packet, if the page sequence
            // is sequential and the packet is from the same bitstream.
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
                return Ok(());
            }
            let mut buffer = [0_u8; 1];
            reader.read_exact(&mut buffer)?;
            if buffer[0] == PAGER_MARKER[marker_found] {
                marker_found += 1;
            } else {
                marker_found = 0;
            }
        }

        Err(BitstreamReadError::UnableToSync)
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

        // Handle unfinished packets. They mostly occur when a packet
        // is bigger than a page would be allowed to be.
        if segment_size != 0 {
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

    fn seek<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        bitstream_serial_number: u32,
        target_granule_position: u64,
    ) -> Result<(), BitstreamReadError> {
        // We assume that packets that spawn multiple pages end in their own page without any now
        // packets in that page.
        // This is currently the behavior the major media mappings (vorbis, opus, flac).
        // Packets only span multiple pages if they are bigger than the maximum allowed
        // packet site.
        self.queued_packets.clear();

        if target_granule_position == u64::MAX {
            reader.seek(SeekFrom::End(0))?;
            return Ok(());
        }

        if target_granule_position == 0 {
            reader.seek(SeekFrom::Start(0))?;
            return Ok(());
        }

        let max_right = reader.seek(SeekFrom::End(0))?;

        let mut left = 0;
        let mut right = max_right;

        let mut target = 0;

        let mut mid: u64;
        'outer: while left < right {
            mid = (left + right) / 2;

            reader.seek(SeekFrom::Start(mid))?;

            let SearchResult {
                packet_start,
                packet_end: _,
                granule_position,
            } = match self.search_next_packet(reader, bitstream_serial_number) {
                Ok(res) => res,
                Err(err) => {
                    handle_eof!(err, break 'outer);
                }
            };

            target = packet_start;

            match granule_position {
                pos if pos < target_granule_position => left = mid.saturating_add(1),
                pos if pos > target_granule_position => right = mid.saturating_sub(1),
                _ => break,
            }

            // If the search volume is small enough, we switch to linear search.
            if (right - left) < 1024 {
                loop {
                    reader.seek(SeekFrom::Start(left))?;
                    let SearchResult {
                        packet_start: _,
                        packet_end,
                        granule_position,
                    } = self.search_next_packet(reader, bitstream_serial_number)?;
                    if granule_position > target_granule_position {
                        target = left;
                        break 'outer;
                    }
                    left = packet_end;
                }
            }
        }
        reader.seek(SeekFrom::Start(target))?;

        Ok(())
    }

    /// Returns the granule position of the next, complete packet. The start and end positions are
    /// the positions that have been searched. A packet can be contained in multiple pages.
    fn search_next_packet<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        bitstream_serial_number: u32,
    ) -> Result<SearchResult, BitstreamReadError> {
        let mut search_start = reader.stream_position()?;
        let mut packet_start = u64::MAX;
        let mut search_buffer = [0_u8; 100];

        'outer: loop {
            let read = reader.read(&mut search_buffer)?;
            if read == 0 {
                return Err(BitstreamReadError::IoError(std::io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "EOF while parsing sync markers",
                )));
            }

            let mut i = 0;
            let mut marker_found = 0;
            loop {
                if i >= read {
                    search_start += 97;
                    reader.seek(SeekFrom::Start(search_start))?;
                    continue 'outer;
                }

                if marker_found == 4 {
                    let page_start = search_start - 4 + u64::try_from(i)?;
                    let page = self.probe_page(reader, page_start)?;

                    if page.bitstream_serial_number != bitstream_serial_number {
                        reader.seek(SeekFrom::Start(page.end))?;
                        continue 'outer;
                    }

                    packet_start = packet_start.min(page.start);

                    if page.granule_position == u64::MAX {
                        reader.seek(SeekFrom::Start(page.end))?;
                        continue 'outer;
                    }

                    return Ok(SearchResult {
                        packet_start,
                        packet_end: page.end,
                        granule_position: page.granule_position,
                    });
                }
                if search_buffer[i] == PAGER_MARKER[marker_found] {
                    marker_found += 1;
                } else {
                    marker_found = 0;
                }

                i += 1;
            }
        }
    }

    fn probe_page<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        page_start: u64,
    ) -> Result<ProbeResult, BitstreamReadError> {
        reader.seek(SeekFrom::Start(page_start))?;
        reader.read_exact(&mut self.page_buffer[HEADER_RANGE])?;

        let granule_position = parse_u64_le(&self.page_buffer[GRANULAR_POSITION_RANGE]);
        let bitstream_serial_number =
            parse_u32_le(&self.page_buffer[BITSTREAM_SERIAL_NUMBER_RANGE]);
        let table_size = usize::from(self.page_buffer[SEGMENT_COUNT_INDEX]);
        let table_start = SEGMENT_TABLE_INDEX;
        let table_end = SEGMENT_TABLE_INDEX + table_size;
        reader.read_exact(&mut self.page_buffer[table_start..table_end])?;

        let mut payload_size = 0;
        for lace in self.page_buffer[table_start..table_end].iter() {
            let bytes = usize::from(*lace);
            match bytes {
                255 => continue,
                _ => {
                    payload_size += bytes;
                }
            }
        }
        let page_end = page_start + u64::try_from(table_start + table_size + payload_size)?;

        Ok(ProbeResult {
            granule_position,
            bitstream_serial_number,
            start: page_start,
            end: page_end,
        })
    }
}

#[derive(Clone, Debug)]
struct SearchResult {
    packet_start: u64,
    packet_end: u64,
    granule_position: u64,
}

#[derive(Clone, Debug)]
struct ProbeResult {
    granule_position: u64,
    bitstream_serial_number: u32,
    start: u64,
    end: u64,
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
        let c = Cursor::new(d);

        let mut br = BitStreamFileReader::new(c);
        let mut packet = Packet::default();
        let res = br.read_packet(&mut packet).unwrap();
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
        let c = Cursor::new(d);

        let mut br = BitStreamFileReader::new(c);
        let mut packet = Packet::default();
        let res = br.read_packet(&mut packet).unwrap();
        assert_eq!(res, ReadStatus::Ok)
    }
}
