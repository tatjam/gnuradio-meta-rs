use std::{
    cell::OnceCell,
    collections::BTreeMap,
    io::{Read, Seek},
    ops::Bound,
};
use thiserror::Error;

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

/// Trait for reading when we are free to seek for headers. Allows some useful
/// features which are impossible if the full header data is not available up front
trait HeaderSeekerReader {
    /// Return which header starts the segment which the sample belongs to, or None
    /// if the sample is out of bounds.
    fn get_header_for_sample(&self, sample: SampleIdx) -> Option<Header>;
}

/// Trait for reading when we may only stream the headers. Even if headers are streamed,
/// it's assumed they are complete!
trait HeaderStreamReader {
    /// Must return the header that's used for the next coming samples. The
    /// trait implementor is responsible for its storage. Return None if no such
    /// header can be found (EOF or nothing loaded yet)
    fn get_current_header(&self) -> Option<Header>;
    /// Must return how many samples remain until the current segment is finished,
    /// or None if unknown.
    fn get_rem_samples_in_segment(&self) -> Option<SampleCount>;
}

struct StreamReaderState {
    cur_header: Option<Header>,
    cur_sample: SampleCount,
    tot_sample: SampleIdx,
}

/// Maps the sample index of the first sample in a header to its header
type HeaderMap = BTreeMap<u64, Header>;

/// Struct representing a GNU radio "File Meta Sink" file, without using the
/// dettached header feature.
/// Least performant if heavy seeking is used, due to non-sequential reading.
struct GnuRadioAttachedHeaderFile {
    stream_state: StreamReaderState,
    // Lazily initialized, just in case only streaming is used (thus the interior mutability)
    header_map: OnceCell<HeaderMap>,
    bin_reader: Box<dyn ReadSeek>,
}

impl HeaderSeekerReader for GnuRadioAttachedHeaderFile {
    fn get_header_for_sample(&self, sample: SampleIdx) -> Option<Header> {
        let map = self.header_map.get_or_init(|| todo!("Implement"));
        // In the header map, each entry maps to the first sample in its header,
        // thus if we find the last entry to be <= to our sample, that's the
        // one we belong to. (This is what bin trees are really efficient at!)
        let last = map
            .range((Bound::Unbounded, Bound::Included(sample)))
            .next_back()?;

        Some(last.1.clone())
    }
}

impl HeaderStreamReader for GnuRadioAttachedHeaderFile {
    fn get_current_header(&self) -> Option<Header> {
        return self.stream_state.cur_header;
    }

    fn get_rem_samples_in_segment(&self) -> Option<SampleCount> {
        let cur_header = self.get_current_header()?;
        let nsamp = cur_header.get_num_samples()?;

        debug_assert!(nsamp >= self.stream_state.cur_sample);
        Some(self.stream_state.cur_sample)
    }
}

/// Struct representing a GNU radio "File Meta Sink" file, using the
/// dettached header feature.
/// Functionally equivalent to GnuRadioAttachedHeaderFile, but faster.
struct GnuRadioDettachedHeaderFile {
    header_reader: Box<dyn ReadSeek>,
    bin_reader: Box<dyn ReadSeek>,
}

/// Struct representing a GNU radio "File Meta Sink" file, but where
/// the binary file is received fully sequentially without
/// the possibility of seeking.
/// Halfway functionality between GnuRadioAttachedHeaderFile and GnuRadioAttachedHeaderStream,
/// but should be highly performant as the binary file is accessed only sequentially.
struct GnuRadioDettachedHeaderStream {
    header_reader: Box<dyn ReadSeek>,
    bin_reader: Box<dyn Read>,
}

/// Struct representing a GNU radio "File Meta Sink" file received stream-wise.
/// Least functionality, but guarantees fully sequential access to the source,
/// which could be more performant.
struct GnuRadioAttachedHeaderStream {
    bin_reader: Box<dyn Read>,
}

#[cfg(test)]
mod core_tests {
    use super::*;

    #[test]
    fn read_complex_samples_attached_file() {}
    #[test]
    fn read_complex_samples_dettached_file() {}
    #[test]
    fn read_complex_samples_attached_stream() {}
    #[test]
    fn read_complex_samples_dettached_stream() {}
}
