use std::io::Write;

use crate::{BitstreamWriteError, PacketType};

// TODO configure the writer to be ablte to decide how big a page should be.
/// Generic OGG bitstream writer.
pub struct BitStreamWriter {}

impl BitStreamWriter {
    /// Writes the given data as a packet into the writer. Packets are not guaranteed to be written
    /// right away. If you want to force them to be written out into the stream, call `flush()`.
    pub fn write_packet<W: Write>(
        _writer: &mut W,
        _packet_data: &[u8],
        _bitstream_serial_number: u64,
        _packet_type: PacketType,
        _granular_position: u64,
    ) -> Result<(), BitstreamWriteError> {
        todo!()
    }

    /// Any remaining packets are written as a page into the writer.
    pub fn flush<W: Write>() {
        todo!()
    }
}
