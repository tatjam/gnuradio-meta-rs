use std::io::{Read, Seek};
use thiserror::Error;

pub trait ReadSeek: Read + Seek {}

/// Trait for reading when we are free to seek for headers. Allows some useful
/// features which are impossible if the full header data is not available up front.
trait HeaderSeekerReader {}

/// Trait for reading when we may only stream the headers.
trait HeaderStreamReader {}

/// Struct representing a GNU radio "File Meta Sink" file, without using the
/// dettached header feature.
/// Least performant if heavy seeking is used, due to non-sequential reading.
struct GnuRadioAttachedHeaderFile {
    bin_reader: Box<dyn ReadSeek>,
}

/// Struct representing a GNU radio "File Meta Sink" file, using the
/// dettached header feature.
/// Functionally equivalent to GnuRadioAttachedHeaderFile.
struct GnuRadioDettachedHeaderFile {
    header_reader: Box<dyn ReadSeek>,
    bin_reader: Box<dyn ReadSeek>,
}

/// Struct representing a GNU radio "File Meta Sink" file, but where
/// the binary file is received stream-wise (say, over the network) without
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
