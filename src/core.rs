use std::{
    cell::OnceCell,
    collections::BTreeMap,
    io::{Read, Seek, SeekFrom},
    ops::Bound,
};

pub trait ReadSeek: Read + Seek {}

/// Refers to a number of samples
type SampleCount = u64;

/// Refers to the position of the sample in the file, the mapping of which to a byte
/// offset can be done with the help of the headers.
type SampleIdx = u64;

/// Represents an offset in samples from another given sample.
type SampleOffset = i64;

#[derive(Copy, Clone)]
pub enum DataType {}

pub struct Error {}

/// Header as read from the GNU radio file
#[derive(Copy, Clone)]
pub struct Header {
    /// Sample rate of the data
    samp_rate: f64,
    /// Reception time of the first sample of the data
    rx_time: (u64, f64),
    /// Size of the item in bytes
    size: u32,
    /// Type of the data
    dtype: DataType,
    /// Is the data complex?
    cplx: bool,
    /// Offset to the first byte of data in this header's segment
    strt: u64,
    /// Size in bytes of the data in this header's segment
    bytes: u64,
}

impl Header {
    fn get_num_samples(&self) -> Option<SampleCount> {
        todo!("Implement");
    }
}

pub struct SampleMeta {}

/// Which qualities of the current segment are guaranteed to be preserved after the seek?
/// When in doubt, use All as most GNU Radio files are a single format and sample rate.
enum SeekPreserve {
    /// Allow seeking into any type of segment
    None,
    /// Allow seeking into segments which have the same format as the current segment
    Format,
    /// Allow seeking into segments which can be converted to the current segment's format
    /// (Implies format)
    Convertability,
    /// Allow seeking into segments which have the same sample rate as the current segment
    SampleRate,
    /// Allow seeking into segments which have the same sample rate, and format which can be converted
    /// to the one of the current segment
    SampleRateAndConvertability,
    /// Allow seeking into segments which have the same sample rate and format as the current one
    All,
    /// Only seek within the current segment. Same guarantees as All, but more restrictive.
    Segment,
}

pub trait HeaderReader {}

/// Similar to Rust's Read + Seek, but obtaining individual samples instead of bytes,
/// and with radio specific functionality (for example, you are guaranteed to never
/// get streams with different sample rates out of this!)
///
/// For maximum performance, it's recommended to only ever read forwards such that all
/// disk access is sequential. This yields maximum speed on most systems.
pub trait SampleReadSeek<T>: HeaderReader {
    /// Fills buf from left to right, at most filling it completely. It will read zero samples
    /// if the next read would imply reading from:
    /// - A segment with type not directly readable to T
    /// - A segment with different sample rate from the one used in previous calls to read
    ///
    /// Returns the number of samples actually read into buf.
    fn read(&mut self, buf: &mut [T]) -> Result<u64, Error> {}

    /// Fills buf from left to right, at most filling it completely. It will read zero samples
    /// if the next read would imply reading from:
    /// - A segment with type inconvertible to T
    /// - A segment with different sample rate from the one used in previous calls to read
    ///
    /// Returns the number of samples actually read into buf.
    fn read_conv(&mut self, buf: &mut [T]) -> Result<u64, Error> {}

    /// Returns metadata that applies to all samples read in the previous call to read
    fn get_last_read_meta(&self) -> Option<SampleMeta> {}

    /// Seeks within the file, preserving certain qualities of the current segment as
    /// given in preserve. Returns the current position in samples from the start of the file, or
    /// errors if the seek could not be performed, leaving the position unmodified.
    fn seek(&mut self, pos: SeekFrom, preserve: SeekPreserve) -> Result<u64, Error> {}

    /// Same as seek, but moving to segment start samples, and pos given in segments.
    /// Returns the current position in samples from the start of the file, or errors if the
    /// seek could not be performed, leaving the position unmodified.
    fn seek_segment(&mut self, pos: SeekFrom, preserve: SeekPreserve) -> Result<u64, Error> {}

    /// Seeks the next segment which has a format that can be converted to `T`, returning the
    /// number of segments skipped, erroring if no such segment can be found.
    fn seek_valid_segment(&mut self) -> Result<u64, Error> {}
}

pub struct AttachedHeader {}

pub struct DettachedHeader {}

#[cfg(test)]
mod core_tests {

    #[test]
    fn read_complex_samples_attached() {}
}
