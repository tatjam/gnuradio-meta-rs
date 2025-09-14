//!
//! # gnuradio-meta
//! This library allows efficiently reading from GNU Radio meta file outputs efficiently.
//! Both attached and dettached headers are supported.
//! In the future, writing data will also be supported.
//!
//! ## Streaming data from GNU Radio
//! This library is not meant to be used while GNU Radio is generating the files. For this purpose it's
//! better to use some of the other sinks like ZeroMQ or similar.
//!
//! ## Crate status
//! * Currently in development, core API may change greatly.
//! * Writing data is currently not implemented nor designed into the API.
//!
//! ## Examples
pub mod core;
