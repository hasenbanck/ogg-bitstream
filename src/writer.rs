use std::convert::TryFrom;
use std::io::Write;

use crate::crc32::crc32;
use crate::{
    WriteError, BITSTREAM_SERIAL_NUMBER_RANGE, BOS_VALUE, CRC32_RANGE, GRANULE_POSITION_RANGE,
    HEADER_TYPE_INDEX, MAX_PAGE_DATA_SIZE, MAX_PAGE_SIZE, PAGER_MARKER, PAGER_MARKER_RANGE,
    PAGE_SEQUENCE_NUMBER_RANGE, SEGMENT_COUNT_INDEX, SEGMENT_TABLE_INDEX,
};

#[derive(Clone, Debug)]
struct StreamState {
    bitstream_serial_number: u32,
    data_buffer: Box<[u8]>,
    data_head: usize,
    packet_sizes: Vec<usize>,
    page_sequence_number: u32,
    granule_position: u64,
    header_type: u8,
}

impl Default for StreamState {
    fn default() -> Self {
        Self {
            bitstream_serial_number: 0,
            data_buffer: vec![0_u8; MAX_PAGE_DATA_SIZE].into_boxed_slice(),
            data_head: 0,
            packet_sizes: Vec::with_capacity(16),
            page_sequence_number: 0,
            granule_position: 0,
            header_type: 0,
        }
    }
}

/// Generic OGG stream writer.
#[derive(Clone, Debug)]
pub struct StreamWriter<W: Write> {
    writer: W,
    stream_states: Vec<StreamState>,
    page_buffer: Box<[u8]>,
}

impl<W: Write> StreamWriter<W> {
    /// Creates a new `StreamWriter`.
    pub fn new(writer: W) -> Self {
        let mut page_buffer = vec![0_u8; MAX_PAGE_SIZE];
        page_buffer[PAGER_MARKER_RANGE].copy_from_slice(&PAGER_MARKER);

        Self {
            writer,
            stream_states: Default::default(),
            page_buffer: page_buffer.into_boxed_slice(),
        }
    }

    /// Consumes the `StreamWriter` and returns the writer.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Starts a new logical stream. Caller needs to provide the first packet, which will be
    /// written to the writer right away.
    pub fn begin_logical_stream(
        &mut self,
        bitstream_serial_number: u32,
        first_packet_data: &[u8],
    ) -> Result<(), WriteError> {
        if self
            .stream_states
            .iter()
            .any(|s| s.bitstream_serial_number == bitstream_serial_number)
        {
            return Err(WriteError::BitstreamAlreadyInitialized);
        }

        if first_packet_data.len() > MAX_PAGE_DATA_SIZE {
            return Err(WriteError::InitialPacketTooBig);
        }

        let mut state = StreamState {
            bitstream_serial_number,
            ..Default::default()
        };

        let size = first_packet_data.len();
        state.packet_sizes.push(size);
        state.data_head = size;
        state.data_buffer[..state.data_head].copy_from_slice(&first_packet_data[..state.data_head]);
        state.header_type = BOS_VALUE;

        write_page(&mut self.writer, &mut self.page_buffer, &mut state)?;

        Ok(())
    }

    /// Ends the logical stream. Caller needs to provide the last packet, which will be
    /// written to the writer right away. Any open pages for this stream will be flushed.
    pub fn end_logical_stream(
        &mut self,
        bitstream_serial_number: u32,
        _last_packet_data: &[u8],
    ) -> Result<(), WriteError> {
        let index = self
            .stream_states
            .iter()
            .enumerate()
            .find(|(_, s)| s.bitstream_serial_number == bitstream_serial_number)
            .map(|(id, _)| id)
            .ok_or(WriteError::UnknownBitstreamSerialNumber)?;

        let state = self.stream_states.remove(index);

        // TODO flush existing data
        // TODO write the last_packet_data (set the header_type)

        todo!()
    }

    /// Writes the given data as a packet into the writer for the specified logical bitstream.
    /// Caller need to begin a stream with `begin_logical_stream` and close it with
    /// `end_logical_stream()`.
    ///
    /// Pages are written once a packet doesn't fit into it's free space, the granule position
    /// changed between packets or `flush()` was called manually.
    ///
    /// Packets will be split into multiple pages if they are bigger than the biggest allowed
    /// data page size of 65_025 B.
    pub fn write_packet(
        _bitstream_serial_number: u64,
        _packet_data: &[u8],
        _granule_position: u64,
    ) -> Result<(), WriteError> {
        // TODO If the current paket doesn't fit on the current page, flush the page and start a new.
        //      Set the continuation flag in that case.

        todo!()
    }

    /// The current page of the logical bitstream is written and a new page is started.
    /// Flushing empty pages is valid.
    pub fn flush(&mut self, bitstream_serial_number: u32) -> Result<(), WriteError> {
        let state = self
            .stream_states
            .iter_mut()
            .find(|s| s.bitstream_serial_number == bitstream_serial_number)
            .ok_or(WriteError::UnknownBitstreamSerialNumber)?;

        write_page(&mut self.writer, &mut self.page_buffer, state)?;

        Ok(())
    }
}

fn write_page<W: Write>(
    writer: &mut W,
    page_buffer: &mut [u8],
    state: &mut StreamState,
) -> Result<(), WriteError> {
    // Write out the segment table.
    let mut segment_count: u8 = 0;
    for packet_size in state.packet_sizes.iter() {
        let full_segments = u8::try_from(packet_size / 255)?;
        for _ in 0..full_segments {
            page_buffer[SEGMENT_TABLE_INDEX + usize::from(segment_count)] = 255;
            segment_count += 1;
        }

        let remainder = u8::try_from(packet_size % 255)?;
        if remainder > 0 {
            page_buffer[SEGMENT_TABLE_INDEX + usize::from(segment_count)] = remainder;
            segment_count += 1;
        }
    }

    // Assemble the page.
    page_buffer[HEADER_TYPE_INDEX] = state.header_type;
    if segment_count == 255 {
        page_buffer[GRANULE_POSITION_RANGE].copy_from_slice(&u64::MAX.to_le_bytes());
    } else {
        page_buffer[GRANULE_POSITION_RANGE].copy_from_slice(&state.granule_position.to_le_bytes());
    }
    page_buffer[BITSTREAM_SERIAL_NUMBER_RANGE]
        .copy_from_slice(&state.bitstream_serial_number.to_le_bytes());
    page_buffer[PAGE_SEQUENCE_NUMBER_RANGE]
        .copy_from_slice(&state.page_sequence_number.to_le_bytes());
    page_buffer[CRC32_RANGE].copy_from_slice(&[0, 0, 0, 0]);
    page_buffer[SEGMENT_COUNT_INDEX] = segment_count;

    let data_start = SEGMENT_TABLE_INDEX + usize::from(segment_count);
    let data_end = data_start + state.data_head;
    page_buffer[data_start..data_end].copy_from_slice(&state.data_buffer[..state.data_head]);

    let crc32 = crc32(&page_buffer[..data_start + state.data_head]);
    page_buffer[CRC32_RANGE].copy_from_slice(&crc32.to_le_bytes());

    // Write out the page and reset the state of the stream.
    writer.write_all(&page_buffer[..data_end])?;

    state.packet_sizes.clear();
    state.data_head = 0;

    state.page_sequence_number += 1;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    #![allow(clippy::unwrap_used)]

    use std::io::Cursor;

    use crate::{parse_u32_le, parse_u64_le, PAGER_MARKER_RANGE, VERSION_INDEX};

    use super::*;

    #[test]
    fn test_begin_streams() {
        let buffer: Vec<u8> = vec![];
        let cursor = Cursor::new(buffer);

        let mut bw = StreamWriter::new(cursor);

        let streams = [
            (12, [0xFF, 0xFF, 0xFF, 0xFF]),
            (42, [0xAA, 0xAA, 0xAA, 0xAA]),
            (99, [0x11, 0x11, 0x11, 0x11]),
            (21, [0x55, 0x44, 0x33, 0x22]),
        ];

        for stream in &streams {
            bw.begin_logical_stream(stream.0, &stream.1).unwrap();
        }

        let cursor = bw.into_inner();
        let buffer = cursor.into_inner();

        let mut offset = 0;
        for stream in &streams {
            assert_eq!(
                &buffer[offset + PAGER_MARKER_RANGE.start..offset + PAGER_MARKER_RANGE.end],
                &PAGER_MARKER
            );
            assert_eq!(buffer[offset + VERSION_INDEX], 0);
            assert_eq!(buffer[offset + HEADER_TYPE_INDEX], BOS_VALUE);
            assert_eq!(
                parse_u64_le(
                    &buffer[offset + GRANULE_POSITION_RANGE.start
                        ..offset + GRANULE_POSITION_RANGE.end]
                ),
                0
            );
            assert_eq!(
                parse_u32_le(
                    &buffer[offset + BITSTREAM_SERIAL_NUMBER_RANGE.start
                        ..offset + BITSTREAM_SERIAL_NUMBER_RANGE.end]
                ),
                stream.0
            );
            assert_eq!(
                parse_u32_le(
                    &buffer[offset + PAGE_SEQUENCE_NUMBER_RANGE.start
                        ..offset + PAGE_SEQUENCE_NUMBER_RANGE.end]
                ),
                0
            );
            assert_eq!(buffer[offset + SEGMENT_COUNT_INDEX], 1);
            assert_eq!(buffer[offset + SEGMENT_TABLE_INDEX], 4);
            assert_eq!(
                &buffer[offset + SEGMENT_TABLE_INDEX + 1..offset + SEGMENT_TABLE_INDEX + 5],
                &stream.1
            );

            offset += SEGMENT_TABLE_INDEX + 5;
        }
    }
}
