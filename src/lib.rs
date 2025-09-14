///!
///! # gnuradio-meta
///! This library allows efficiently reading from GNU Radio meta file outputs efficiently, whether they are
///! handled as entire, seekable files, or as streams of bytes. Both attached and dettached headers are supported,
///! and it's also possible to use a seekable header file alongside a stream of binary data.
///! In the future, writing data will also be supported.
///!
///! ## Crate status
///! * Currently in development, core API may change greatly.
///! * Writing data is currently not implemented nor designed into the API.
///!
///! ## Examples
pub mod core;
