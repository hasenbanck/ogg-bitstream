use std::io::Write;

use crate::BitstreamWriteError;

/// The type of the packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketType {
    /// The packets marks the beginning of a sequence.
    Bos,
    /// The packets is inside of a sequence.
    Normal,
    /// The packet marks the end of a sequence.
    Eos,
}

/// Generic OGG bitstream writer.
pub struct BitStreamWriter {}

impl BitStreamWriter {
    /// Writes the given data as a packet into the writer. Packets are only written when a page
    /// is full or a `flush()` was called.
    pub fn write_packet<W: Write>(
        _writer: &mut W,
        _packet_data: &[u8],
        _bitstream_serial_number: u64,
        _granular_position: u64,
        _packet_type: PacketType,
    ) -> Result<(), BitstreamWriteError> {
        todo!()
    }

    /// Any remaining packets are written as a page into the writer.
    pub fn flush<W: Write>() {
        todo!()
    }
}
