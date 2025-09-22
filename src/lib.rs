//! # gnuradio-meta
//! This library allows reading from GNU Radio meta file outputs efficiently, including handling of headers in order
//! to extract stream tags.
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
//!
//! ### Getting system time
//!
//! Sometimes, you may be interested in getting a precision timestamp for your samples. Your computer's time can be
//! remarkably precise. Using an NTP system based on GPS, you could achieve
//! [precision well below 10us with respect to UTC in your system timer](https://scottstuff.net/posts/2025/05/19/ntp-limits/).
//!
//! The timestamp read from the header is not really appropiate for time-keeping samples, as it's relative to the first
//! sample, and thus if you care about the absolute epoch of a sample it becomes useless.
//! Nonetheless, you can easily implement a stream tag generated ocasionally to map your samples to a moment in
//! real time, with reasonable precision.
//! To do so, create a Python Module in GNU Radio with id `timemark` and the following code:
//!
//! ```python3
//! from time import time_ns
//! import pmt
//! def get_time_pair():
//!     now = time_ns()
//!     now_secs = int(now / 1e9)
//!     now_frac = now / 1e9 - now_secs
//!     pmt_pair = pmt.cons(pmt.from_uint64(now_secs), pmt.from_double(now_frac))
//!     return pmt.dict_add(pmt.make_dict(), pmt.intern("timemark"), pmt_pair)
//! ```
//! You can then set the "Extra dict." attribute of the GNU Radio File Meta Sink to `timemark.get_time_pair()`,
//! which will add this tag to every header generated (by default every 1M samples), including the first one.
//! You can then read it from each header as a Timestamp value in Rust.
//!
pub mod core;
mod header;
mod pmt;
