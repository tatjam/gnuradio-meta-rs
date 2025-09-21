use std::io::{Read, Seek, SeekFrom};
use thiserror::Error;

/// A date-time with 64 bits for the second and 64 bits for the fractional part,
/// allowing accurate time-keeping in seconds regardless of origin point, maintaining
/// sub nano-second precision at any date, and wrapping around in billions of years
/// into the future.
///
/// Note that using a f32 for timestamps is inappropiate if you need them relative
/// to UNIX epoch or similar, as nowadays the precision is way less than a second.
/// Similarly, f64 are not appropiate if you need high precision. As of 2025, the
/// double precision for a UNIX timestamp is down to 0.3us, which may not be good
/// enough in some high precision radio applications, and will only get worse!
///
/// If you only need timestamps relative to the start of the file, a f32 or f64 is
/// probably fine, but this is how GNU Radio gives the data.
pub type Timestamp = fixed::FixedI128<fixed::types::extra::U64>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DataType {}

impl DataType {
    pub fn reads_directly_to<T>(&self) -> bool {
        todo!("Implement");
    }

    pub fn converts_to<T>(&self) -> bool {
        todo!("Implement");
    }

    pub fn converts_to_dtype(&self, other: DataType) -> bool {
        todo!("Implement");
    }

    pub fn convert_to<T>(&self, bytes: &[u8]) -> T {
        todo!("Implement");
    }
}

#[derive(Error, Debug)]
pub enum MetaFileError {
    #[error("Reader I/O error")]
    IoError(#[from] std::io::Error),
}

/// Header as read from the GNU radio file
#[derive(PartialEq, Debug, Clone)]
pub struct Header {
    /// Sample rate of the data
    samp_rate: f64,
    /// Duration of a sample, computed from samp_rate
    samp_dur: f64,
    /// Reception time of the first sample of the data, relative to first sample
    rx_time: Timestamp,
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

    /// Absolute position of the first byte of the data from the start of the file,
    /// computed by ourselves
    abs_pos: u64,
}

/// Which qualities of the current segment are guaranteed to be preserved after the seek?
/// When in doubt, use All as most GNU Radio files are a single format and sample rate.
#[derive(PartialEq, Eq)]
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

impl SeekPreserve {
    /// Returns true if the seek guarantees that the resulting segment data type is equal to the
    /// one before the seek
    fn preserves_format(&self) -> bool {
        *self == SeekPreserve::Format
            || *self == SeekPreserve::All
            || *self == SeekPreserve::Segment
    }

    /// Returns true if the seek guarantees that the resulting segment data type can be converted
    /// to the one before the seek
    fn preserves_convertability(&self) -> bool {
        *self != SeekPreserve::None && *self != SeekPreserve::SampleRate
    }

    /// Returns true if the seek guarantees that the resulting segment sample rate is equal to the
    /// one before the seek
    fn preserves_samplerate(&self) -> bool {
        *self != SeekPreserve::None
            && *self != SeekPreserve::Format
            && *self != SeekPreserve::Convertability
    }
}

impl Header {
    fn get_num_samples(&self) -> u64 {
        todo!("Implement");
    }

    /// Returns the expected reception time of sample at offset `sample` (which
    /// may be outside the header just fine, or even negative) assuming the sample rate is held
    /// constant until said offset.
    fn get_sample_time(&self, sample: i64) -> Timestamp {
        todo!("Implement");
    }

    /// Gets the duration of a sample at the sample rate of the header
    fn get_sample_duration(&self) -> f64 {
        return self.samp_dur;
    }

    fn is_compatible_with(&self, other: &Header, preserve: SeekPreserve) -> bool {
        if preserve.preserves_samplerate() && other.samp_rate != self.samp_rate {
            return false;
        }
        if preserve.preserves_format() && other.dtype != self.dtype {
            return false;
        }
        if preserve.preserves_convertability() && !other.dtype.converts_to_dtype(self.dtype) {
            return false;
        }

        true
    }

    /// Returns true if the first sample of this header is received at most with a time error
    /// of 0.1 * other.get_sample_duration(), to account for floating point errors.
    fn is_continuation_of(&self, other: &Header) -> bool {
        let last_sample_t = if other.get_num_samples() == 0 {
            other.rx_time
        } else {
            other.get_sample_time(other.get_num_samples() as i64 - 1)
        };
        let diff = self.rx_time.abs_diff(last_sample_t).to_num::<f64>();
        diff <= 0.1 * other.get_sample_duration()
    }

    fn get_sample_pos_of_byte(&self, byte: u64) -> u64 {
        todo!("Implement");
    }
}

pub struct StreamTag {}

pub struct SampleMeta {
    /// Sample rate of the data read
    samp_rate: f64,
    /// Reception time of the first sample read
    rx_time: Timestamp,
}

/// This trait allows accessing headers for both attached and dettached files using a common interface.
pub trait HeaderReader {
    /// Returns the header_num header, with 0 being the first header of the file, if it exists.
    /// If it needs to seek in the binary file, it must restore the seeker at the end.
    fn get_header_at(&self, header_num: usize) -> Result<Option<&Header>, MetaFileError>;

    /// Gets the header applicable to a byte in the binary file (byte) or None if non-existent.
    fn get_header_for_byte(&self, byte: u64) -> Result<Option<&Header>, MetaFileError>;
}

pub trait RawSampleReader: Read + Seek {
    /// Read samples from the binary into the target buffer, performing no conversion, and assuming
    /// all bytes read are valid samples that are directly convertible to T (i.e. readable with endianness change)
    fn read_raw<T>(&mut self, tgt: &mut [T]) -> Result<u64, MetaFileError>;

    /// Read samples from the binary into the target buffer, performing conversion from the given type
    /// to be assumed to be stored in the binary samples
    fn read_raw_conv<T>(&mut self, tgt: &mut [T], dtype: DataType) -> Result<u64, MetaFileError>;
}

/// Similar to Rust's Read + Seek, but obtaining individual samples instead of bytes,
/// and with radio specific functionality (for example, you are guaranteed to never
/// get streams with different sample rates, or with time jumps, if you use read())
///
/// For maximum performance, it's recommended to only ever read forward such that all
/// disk access is sequential. This should yield maximum speed on most systems.
pub trait SampleReadSeek {
    fn get_header_reader(&self) -> &mut impl HeaderReader;
    fn get_sample_reader(&self) -> &mut impl RawSampleReader;

    /// Gets the header that the last read sample belonged to, or None if no samples
    /// have been read yet, or a seek has been performed.
    fn get_last_read_header(&self) -> Result<Option<&Header>, MetaFileError> {
        let pos = self.get_sample_reader().stream_position()?;
        return self.get_header_reader().get_header_for_byte(pos);
    }

    fn get_last_read_rx_time(&mut self) -> Option<Timestamp> {
        todo!("Implement");
        /*
        let offset = self.get_last_read_offset_in_header()?;
        let header = self.get_last_read_header()?;
        header.get_sample_time(offset);
        */
    }

    #[doc(hidden)]
    fn get_last_and_applicable_header(&self) -> Result<Option<(&Header, &Header)>, MetaFileError> {
        let last_header = match self.get_last_read_header()? {
            None => return Ok(None), // EOF of empty file
            Some(v) => v,
        };

        let appl_header = if last_header.abs_pos + last_header.bytes
            <= self.get_sample_reader().stream_position()?
        {
            // We finished the last segment, seek to next one
            self.get_sample_reader().seek(SeekFrom::Current(1))?;
            let out = match self
                .get_header_reader()
                .get_header_for_byte(self.get_sample_reader().stream_position()?)?
            {
                None => return Ok(None), // EOF achieved
                Some(v) => v,
            };
            self.get_sample_reader()
                .seek(SeekFrom::Start(out.abs_pos))?;
            out
        } else {
            // We keep reading from last segment
            last_header
        };

        Ok(Some((last_header, appl_header)))
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
    ///
    /// If an error is returned, the buffer may have been modified!
    fn read_samples<T>(&self, buf: &mut [T]) -> Result<u64, MetaFileError> {
        let mut num_read: u64 = 0;

        while num_read < buf.len() as u64 {
            let (last_header, appl_header) = match self.get_last_and_applicable_header()? {
                Some(v) => v,
                None => break, // EOF or empty file
            };

            if !appl_header.dtype.reads_directly_to::<T>() {
                break; // Not directly readable to T, stop reading
            }

            if appl_header != last_header {
                if !appl_header.is_compatible_with(last_header, SeekPreserve::All) {
                    break; // Something is different about the new header, stop reading
                }

                if !appl_header.is_continuation_of(last_header) {
                    break; // The segment had a time discontinuity, stop reading
                }
            }

            let buff_remain = buf.len() as u64 - num_read;

            let cur_sample =
                appl_header.get_sample_pos_of_byte(self.get_sample_reader().stream_position()?);
            let samps_remain = appl_header.get_num_samples() - cur_sample;

            let to_read = buff_remain.min(samps_remain);
            let start = num_read as usize;
            let end = start + to_read as usize;

            num_read += self
                .get_sample_reader()
                .read_raw::<T>(&mut buf[start..end])?;
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
    fn read_conv<T>(&mut self, buf: &mut [T]) -> Result<u64, MetaFileError> {
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
    fn seek(&mut self, pos: SeekFrom, preserve: SeekPreserve) -> Result<u64, MetaFileError> {
        todo!("Implement");
    }

    /// Same as seek, but moving to segment start samples, and pos given in segments.
    /// Returns the current position in samples from the start of the file, or errors if the
    /// seek could not be performed, leaving the position unmodified.
    fn seek_segment(
        &mut self,
        pos_seg: SeekFrom,
        preserve: SeekPreserve,
    ) -> Result<u64, MetaFileError> {
        todo!("Implement");
    }

    /// Seeks the next segment which has a format that can be converted to `T`, returning the
    /// number of segments skipped, erroring if no such segment can be found.
    fn seek_valid_segment(&mut self) -> Result<u64, MetaFileError> {
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
