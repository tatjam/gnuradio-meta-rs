use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::rc::Rc;

use crate::pmt::{Tag, parse, parse_maybe_eof};
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

#[derive(Default)]
pub struct HeaderStorage {
    /// Maps a byte in the binary file to the header that starts at that byte, either
    /// because it's stored there, or because the first byte of that header's segment is there.
    store: BTreeMap<u64, Header>,
}

impl HeaderStorage {
    /// Gets the header applicable to a byte in the binary file (byte) or None if not loaded.
    fn get_header_for_byte(&self, byte: u64) -> Option<&Header> {
        todo!();
    }

    fn add_header_for_byte(&mut self, byte: u64, header: Header) {
        // Check that all headers previous to this one have been loaded, or none
        // previous to it have been loaded, so the indexing logic works
        self.store.insert(byte, header);
    }
}

/// Note all of these can be "complex", which duplicates each entry as a complex number,
/// and makes them directly convertible to Complex<x>.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DataType {
    /// Directly convertible to i8
    Byte,
    /// Directly convertible to i16
    Short,
    /// Directly convertible to i32
    Int,
    // Long (not possible from GNU Radio)
    // LongLong, (not possible from GNU Radio)
    /// Directly convertible to f32
    Float,
    /// Directly convertible to f64
    Double,
}

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
    #[error("PMT parser error")]
    ParseError(#[from] crate::pmt::ParseError),
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

    extra_dict: Rc<Tag>,

    /// Absolute position of the first byte of the data from the start of the file,
    /// computed by ourselves
    abs_pos: u64,

    /// Absolute position of the first byte of the HEADER in the file (either attached or dettached),
    /// computed by ourselves
    pos_in_file: u64,
}

/// Which qualities of the current segment are guaranteed to be preserved after the seek?
/// When in doubt, use All as most GNU Radio files are a single format and sample rate.
#[derive(PartialEq, Eq)]
pub enum SeekPreserve {
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
        self.samp_dur
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

    fn from_tags(tag: Tag, extra: Tag) -> Result<Header, MetaFileError> {
        todo!();
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
    fn get_header_storage_mut(&mut self) -> &mut HeaderStorage;
    fn get_header_storage(&self) -> &HeaderStorage;

    /// Load the next header from the file. start_byte is the first byte of said header in the binary file
    /// (thus only used in AttachedHeader mode!). Return None if no more to read.
    fn load_next_header(&mut self, start_byte: u64) -> Result<Option<Header>, MetaFileError>;

    #[doc(hidden)]
    fn get_first_byte_of_next_header_to_read(&mut self) -> u64 {
        // We are guaranteed to have the last header read, so simply get the byte after
        // the last data in the previous (last loaded) header
        let last = match self.get_header_storage_mut().store.last_entry() {
            None => return 0, // No headers are loaded, this is the first byte of the file either way
            Some(v) => v,
        };

        // TODO: bytes may be wrong!

        last.get().abs_pos + last.get().bytes + 1
    }

    fn get_header_for_byte(&mut self, byte: u64) -> Result<Option<Header>, MetaFileError> {
        if let Some(v) = self.get_header_storage().get_header_for_byte(byte) {
            return Ok(Some(v.clone()));
        }

        // Not loaded! We need to get the first byte of the header, as 'byte' may be at any point
        // in the segment. Note that headers are always loaded "left-to-right", so this may load
        // a whole bunch of headers.
        loop {
            let first_byte = self.get_first_byte_of_next_header_to_read();
            if first_byte >= byte {
                // It should have already been loaded
                return Ok(self.get_header_storage().get_header_for_byte(byte).cloned());
            }
            if let Some(v) = self.load_next_header(first_byte)? {
                self.get_header_storage_mut()
                    .add_header_for_byte(first_byte, v);
            } else {
                // We reached EOF...
                break;
            }
        }

        // ...out of bounds byte
        Ok(None)
    }
}

fn read_raw<T>(reader: &mut impl Read, target: &mut [T]) -> Result<u64, MetaFileError> {
    todo!();
}

/// Similar to Rust's Read + Seek, but obtaining individual samples instead of bytes,
/// and with radio specific functionality (for example, you are guaranteed to never
/// get streams with different sample rates, or with time jumps, if you use read())
///
/// For maximum performance, it's recommended to only ever read forward such that all
/// disk access is sequential. This should yield maximum speed on most systems.
pub trait SampleReadSeek {
    fn get_header_reader_mut(&mut self) -> &mut impl HeaderReader;
    fn get_sample_reader_mut(&mut self) -> &mut (impl Read + Seek);

    /// Gets the header that the last read sample belonged to, or None if no samples have been read
    /// yet, or a seek has been performed.
    fn get_last_read_header(&mut self) -> Result<Option<Header>, MetaFileError> {
        let pos = self.get_sample_reader_mut().stream_position()?;
        self.get_header_reader_mut().get_header_for_byte(pos)
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
    fn get_last_and_applicable_header(
        &mut self,
    ) -> Result<Option<(Header, Header)>, MetaFileError> {
        let last_header = match self.get_last_read_header()? {
            None => return Ok(None), // EOF of empty file
            Some(v) => v,
        };

        let appl_header = if last_header.abs_pos + last_header.bytes
            <= self.get_sample_reader_mut().stream_position()?
        {
            // We finished the last segment, seek to next one
            self.get_sample_reader_mut().seek(SeekFrom::Current(1))?;
            let cur_pos = self.get_sample_reader_mut().stream_position()?;
            let out = match self.get_header_reader_mut().get_header_for_byte(cur_pos)? {
                None => return Ok(None), // EOF achieved
                Some(v) => v,
            };
            self.get_sample_reader_mut()
                .seek(SeekFrom::Start(out.abs_pos))?;
            out
        } else {
            // We keep reading from last segment
            last_header.clone()
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
    fn read_samples<T>(&mut self, buf: &mut [T]) -> Result<u64, MetaFileError> {
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
                if !appl_header.is_compatible_with(&last_header, SeekPreserve::All) {
                    break; // Something is different about the new header, stop reading
                }

                if !appl_header.is_continuation_of(&last_header) {
                    break; // The segment had a time discontinuity, stop reading
                }
            }

            let buff_remain = buf.len() as u64 - num_read;

            let cur_sample =
                appl_header.get_sample_pos_of_byte(self.get_sample_reader_mut().stream_position()?);
            let samps_remain = appl_header.get_num_samples() - cur_sample;

            let to_read = buff_remain.min(samps_remain);
            let start = num_read as usize;
            let end = start + to_read as usize;

            num_read += read_raw(self.get_sample_reader_mut(), &mut buf[start..end])?;
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

pub struct AttachedHeader<T: Read + Seek> {
    header_storage: HeaderStorage,
    file: T,
}

impl<T: Read + Seek> AttachedHeader<T> {
    fn new(file: T) -> AttachedHeader<T> {
        AttachedHeader {
            header_storage: Default::default(),
            file,
        }
    }
}

impl<T: Read + Seek> HeaderReader for AttachedHeader<T> {
    fn get_header_storage_mut(&mut self) -> &mut HeaderStorage {
        &mut self.header_storage
    }
    fn get_header_storage(&self) -> &HeaderStorage {
        &self.header_storage
    }

    fn load_next_header(&mut self, start_byte: u64) -> Result<Option<Header>, MetaFileError> {
        todo!()
    }
}

impl<T: Read + Seek> SampleReadSeek for AttachedHeader<T> {
    fn get_header_reader_mut(&mut self) -> &mut impl HeaderReader {
        self
    }

    fn get_sample_reader_mut(&mut self) -> &mut (impl Read + Seek) {
        &mut self.file
    }
}

pub struct DettachedHeader<B: Read + Seek, H: Read + Seek> {
    header_storage: HeaderStorage,
    header_file: B,
    binary_file: H,
}

impl<B: Read + Seek, H: Read + Seek> HeaderReader for DettachedHeader<B, H> {
    fn get_header_storage_mut(&mut self) -> &mut HeaderStorage {
        &mut self.header_storage
    }

    fn get_header_storage(&self) -> &HeaderStorage {
        &self.header_storage
    }

    fn load_next_header(&mut self, start_byte: u64) -> Result<Option<Header>, MetaFileError> {
        // header_file seek is always at the next header, so we can simply
        let header_tag = match parse_maybe_eof(&mut self.header_file) {
            Ok(Some(v)) => v,
            Ok(None) => return Ok(None),
            Err(e) => return Err(MetaFileError::ParseError(e)),
        };
        let extra = parse(&mut self.header_file)?;
        let header = Header::from_tags(header_tag, extra)?;
        Ok(Some(header))
    }
}

impl<B: Read + Seek, H: Read + Seek> SampleReadSeek for DettachedHeader<B, H> {
    fn get_header_reader_mut(&mut self) -> &mut impl HeaderReader {
        self
    }

    fn get_sample_reader_mut(&mut self) -> &mut (impl Read + Seek) {
        &mut self.binary_file
    }
}

#[cfg(test)]
mod core_tests {
    use super::*;
    use std::fs::File;

    /// Returns the binary file (always) and the header file if it exists
    fn get_or_run_gnuradio(file: &'static str) -> (File, Option<File>) {
        use std::path::Path;
        use std::process::Command;

        let src_path = format!("test_files/{}.grc", file);
        let src_path_py = format!("target/test_files/{}.py", file);
        let dst_path = format!("target/test_files/{}", file);
        // not always generated
        let dst_path_hdr = format!("target/test_files/{}.grh", file);

        // Compile the file
        let cout = Command::new("grcc")
            .args(&[src_path.as_str(), "-o", "target/test_files/"])
            .output()
            .expect(format!("failed to run GNU radio compiler on {}", src_path).as_str());
        if !cout.status.success() {
            panic!(
                "GNU radio compiler failed on file {}, error: {}",
                src_path,
                String::from_utf8(cout.stderr).unwrap()
            )
        }

        let out = Command::new("python3")
            .args(&[src_path_py.as_str(), "--out", dst_path.as_str()])
            .output()
            .expect(format!("failed to run GNU radio on {}", file).as_str());
        if !out.status.success() {
            panic!(
                "GNU radio failed on file {}, error: {}",
                file,
                String::from_utf8(out.stderr).unwrap(),
            )
        }

        (File::open(dst_path).unwrap(), File::open(dst_path_hdr).ok())
    }

    #[test]
    fn read_byte_samples_attached() {
        let (file, _) = get_or_run_gnuradio("bytes_increasing");
        let mut reader = AttachedHeader::new(file);
        let mut samples: [u8; 256] = [0; 256];

        let num_read = reader.read_samples(&mut samples).unwrap();
        // Basic sample reading checks
        assert_eq!(num_read, 256);
        for i in 0..256 {
            assert_eq!(samples[i], i as u8);
        }

        // Public header interface
        let header = reader.get_last_read_header();

        // Check sane internal state for headers

        // Further reads should return nothing
    }
}
