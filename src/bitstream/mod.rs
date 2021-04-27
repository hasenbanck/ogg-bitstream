pub use read_error::BitstreamReadError;
pub use reader::{BitStreamReader, ReadStatus};
pub use write_error::BitstreamWriteError;
pub use writer::BitStreamWriter;

mod read_error;
mod reader;
mod write_error;
mod writer;

/// A packet inside an OGG stream.
pub struct Packet {
    /// The data of the packet.
    data: Vec<u8>,
    /// The sequential number of the packet.
    packet_num: u64,
    /// The granular position of the last sample (`granule`) in the packet.
    granular_position: u64,
    /// The type of the packet.
    packet_type: PacketType,
}

/// The type of the packet.
pub enum PacketType {
    /// The packets marks the beginning of a sequence.
    Bos,
    /// The packets is inside of a sequence.
    Normal,
    /// The packet marks the end of a sequence.
    Eos,
}
