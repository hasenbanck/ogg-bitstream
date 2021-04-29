#[cfg(feature = "decoder")]
pub use read_error::*;
#[cfg(feature = "encoder")]
pub use write_error::*;

#[cfg(feature = "opus")]
mod opus;
#[cfg(feature = "decoder")]
mod read_error;
#[cfg(feature = "vorbis")]
mod vorbis;
#[cfg(feature = "encoder")]
mod write_error;

#[cfg(feature = "decoder")]
/// Generic stream seeker trait. Used to abstract the seeking inside streams.
pub trait StreamSeeker {
    /// Seeks to the first packet after the given granule position.
    fn seek(granule_position: u64) -> Result<(), MediaReadError>;
}

#[cfg(all(feature = "opus", feature = "decoder"))]
/// Generic stream reader trait. Used to abstract the reading of frames.
pub trait StreamReader {
    /// Decodes the next packet of the stream and writes the frames inside the given vector.
    ///
    /// Returns `false` when we reached the end of the stream.
    fn decode_packet<S: Sample>(&self, frames: &mut Vec<Vec<S>>) -> Result<bool, MediaReadError>;

    /// Decodes the next packet of the stream and writes the frames inside the given vector with
    /// samples a `f32`.
    ///
    /// Returns `false` when we reached the end of the stream.
    fn decode_packet_f32(&self, frames: &mut Vec<Vec<f32>>) -> Result<bool, MediaReadError>;

    /// Decodes all packet of the stream and writes the frames inside the given vector.
    ///
    /// Returns `false` when we reached the end of the stream.
    fn decode_packet_all<S: Sample>(
        &self,
        frames: &mut Vec<Vec<S>>,
    ) -> Result<bool, MediaReadError>;

    /// Decodes all packet of the stream and writes the frames inside the given vector with
    /// samples a `f32`.
    ///
    /// Returns `false` when we reached the end of the stream.
    fn decode_packet_all_f32(&self, frames: &mut Vec<Vec<f32>>) -> Result<bool, MediaReadError>;
}

#[cfg(all(feature = "vorbis", feature = "decoder"))]
/// Generic stream reader trait for allocating readers. Used to abstract the reading of frames.
pub trait AllocatingStreamReader {
    /// Decodes the next packet of the stream and writes the frames into a new vector.
    ///
    /// Returns `None` when we reached the end of the stream.
    fn decode_packet<S: Sample>(&self) -> Result<Option<Vec<Vec<S>>>, MediaReadError>;

    /// Decodes the next packet of the stream and writes the frames into a new vector with
    /// samples a `f32`.
    ///
    /// Returns `None` when we reached the end of the stream.
    fn decode_packet_f32(&self) -> Result<Option<Vec<Vec<f32>>>, MediaReadError>;

    /// Decodes all packet of the stream and writes the frames into a new vector.
    ///
    /// Returns `false` when we reached the end of the stream.
    fn decode_packet_all<S: Sample>(&self) -> Result<Option<Vec<Vec<S>>>, MediaReadError>;

    /// Decodes all packet of the stream and writes the frames into a new vector with
    /// samples a `f32`.
    ///
    /// Returns `false` when we reached the end of the stream.
    fn decode_packet_all_f32(&self) -> Result<Option<Vec<Vec<f32>>>, MediaReadError>;
}

#[cfg(feature = "encoder")]
/// Generic stream writer trait. Used to abstract the writing of frames.
pub trait StreamWriter {
    /// Writes the given frames into the stream.
    fn write_frames<S: Sample>(&self, frames: &[Vec<S>]) -> Result<(), MediaWriteError>;

    /// Writes the given frames of `f32` samples into the stream.
    fn write_frames_f32(&self, frames: &[Vec<f32>]) -> Result<(), MediaWriteError>;
}

#[cfg(any(feature = "encoder", feature = "decoder"))]
/// Generic sample trait.
pub trait Sample {
    /// Implementation need to convert the given `f32` into it's desired storage format.
    fn from_f32(f: f32) -> Self;
}

#[cfg(any(feature = "encoder", feature = "decoder"))]
impl Sample for f32 {
    fn from_f32(f: f32) -> Self {
        f
    }
}

#[cfg(any(feature = "encoder", feature = "decoder"))]
#[allow(clippy::as_conversions)]
impl Sample for i16 {
    fn from_f32(f: f32) -> Self {
        let mut x: f32 = f * 32_768.0;
        x = x.max(-32_768.0);
        x = x.min(32_767.0);
        x as i16
    }
}

#[cfg(any(feature = "encoder", feature = "decoder"))]
#[allow(clippy::as_conversions)]
impl Sample for u16 {
    fn from_f32(f: f32) -> Self {
        let mut x: f32 = (f * 32_768.0) + 32_768.0;
        x = x.max(0.0);
        x = x.min(65_535.0);
        x as u16
    }
}
