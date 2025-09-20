use std::{
    cell::OnceCell,
    collections::BTreeMap,
    io::{Read, Seek, SeekFrom},
};

use crate::rxtime::RxTime;

pub trait ReadSeek: Read + Seek {}

/// Refers to a number of samples
type SampleCount = u64;

/// Refers to the position of the sample in the file, the mapping of which to a byte
/// offset can be done with the help of the headers.
type SampleIdx = u64;

/// Represents an offset in samples from another given sample.
type SampleOffset = i64;

#[derive(Copy, Clone, PartialEq)]
pub enum DataType {}

impl DataType {
    pub fn reads_directly_to<T>(&self) -> bool {
        todo!("Implement");
    }

    pub fn converts_to<T>(&self) -> bool {
        todo!("Implement");
    }

    pub fn convert_to<T>(&self, bytes: &[u8]) -> T {
        todo!("Implement");
    }
}

pub struct Error {}

/// Header as read from the GNU radio file
#[derive(Copy, Clone, PartialEq)]
pub struct Header {
    /// Sample rate of the data
    samp_rate: f64,
    /// Reception time of the first sample of the data
    rx_time: RxTime,
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

    /// Returns the expected reception time of sample at offset `sample` (which
    /// may be outside the header just fine) assuming the sample rate is held
    /// constant until said offset.
    fn get_sample_time(&self, sample: i64) -> RxTime {
        todo!("Implement");
    }
}

pub struct StreamTag {}

pub struct SampleMeta {
    /// Sample rate of the data read
    samp_rate: f64,
    /// Reception time of the first sample read
    rx_time: RxTime,
}

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

/// This trait allows accessing headers for both attached and dettached files using a common interface.
pub trait HeaderReader {}

/// Similar to Rust's Read + Seek, but obtaining individual samples instead of bytes,
/// and with radio specific functionality (for example, you are guaranteed to never
/// get streams with different sample rates, or with time jumps, out of this!)
///
/// For maximum performance, it's recommended to only ever read forwards such that all
/// disk access is sequential. This yields maximum speed on most systems.
pub trait SampleReadSeek<T>: HeaderReader {
    /// Gets the header that the next sample that will be read belongs to, or None if
    /// EOF has been reached.
    fn get_next_read_header(&mut self) -> Option<Header>;

    /// Gets the header that the last read sample belonged to, or None if no samples
    /// have been read yet, or a seek has been performed.
    fn get_last_read_header(&mut self) -> Option<Header>;

    /// Gets the sample position relative to the first sample of the header the
    /// last read sample belongs to. Return None if no sample has been read yet, or a seek
    /// has been performed.
    fn get_last_read_offset_in_header(&mut self) -> Option<u64>;

    fn get_last_read_rx_time(&mut self) -> Option<RxTime> {
        todo!("Implement");
        /*
        let offset = self.get_last_read_offset_in_header()?;
        let header = self.get_last_read_header()?;
        header.get_sample_time(offset);
        */
    }

    /// Fills buf from left to right, at most filling it completely. It will stop reading samples
    /// if the next read would imply reading from:
    /// - A segment with type not directly readable to T
    /// - A segment with different sample rate from the one used in previous calls to read
    /// - A segment with a jump in time with respect to the last sample of the previous segment
    ///
    /// Returns the number of samples actually read into buf.
    /// This function will never perform conversion, so it's the fastest possible as it
    /// will simply copy from the source file to the destination array.
    fn read(&mut self, buf: &mut [T]) -> Result<u64, Error> {
        let mut num_read: u64 = 0;

        while num_read < buf.len() as u64 {
            let next_read_header = match self.get_next_read_header() {
                Some(data) => data,
                None => break, // EOF reached!
            };

            let current_header = match self.get_last_read_header() {
                Some(data) => data,
                None => next_read_header, // First read from file, for logic assume current == next header
            };

            if next_read_header != current_header {
                if !next_read_header.dtype.reads_directly_to::<T>() {
                    break; // Not directly readable to T, stop reading
                }

                if next_read_header.samp_rate != current_header.samp_rate {
                    break; // Sample rate changed, not allowed, stop reading
                }

                todo!("Implement!");
                /*
                if next_read_header.rx_time != next_expected_rx_time {
                    break; // The next header supposed a time skip, not allowed, stop reading.
                }
                */
            }
        }

        Ok(num_read)
    }

    /// Fills buf from left to right, at most filling it completely. It will stop reading samples
    /// if the next read would imply reading from:
    /// - A segment with type inconvertible to T
    /// - A segment with different sample rate from the one used in previous calls to read
    /// - A segment with a jump in time with respect to the last sample of the previous segment
    ///
    /// Returns the number of samples actually read into buf.
    /// This function may convert if neccesary, and is thus expected to be slightly slower
    /// than read.
    fn read_conv(&mut self, buf: &mut [T]) -> Result<u64, Error> {
        todo!("Implement");
    }

    /// Returns metadata that applies to all samples read in the previous call to read.
    fn get_last_read_meta(&self) -> Option<SampleMeta> {
        todo!("Implement");
    }

    /// Returns stream tags applicable to samples in previous call to read
    fn get_last_read_tags(&self) -> Vec<StreamTag> {
        // DO NOT ALLOCATE DURING READ, instead just store first and last sample and
        // perform the "complicated" tag extraction here
        todo!("Implement");
    }

    /// Seeks within the file, preserving certain qualities of the current segment as
    /// given in preserve. Returns the current position in samples from the start of the file, or
    /// errors if the seek could not be performed, leaving the position unmodified.
    fn seek(&mut self, pos: SeekFrom, preserve: SeekPreserve) -> Result<u64, Error> {
        todo!("Implement");
    }

    /// Same as seek, but moving to segment start samples, and pos given in segments.
    /// Returns the current position in samples from the start of the file, or errors if the
    /// seek could not be performed, leaving the position unmodified.
    fn seek_segment(&mut self, pos_seg: SeekFrom, preserve: SeekPreserve) -> Result<u64, Error> {
        todo!("Implement");
    }

    /// Seeks the next segment which has a format that can be converted to `T`, returning the
    /// number of segments skipped, erroring if no such segment can be found.
    fn seek_valid_segment(&mut self) -> Result<u64, Error> {
        todo!("Implement");
    }
}

pub struct AttachedHeader {}

pub struct DettachedHeader {}

#[cfg(test)]
mod core_tests {

    #[test]
    fn read_complex_samples_attached() {}
}
