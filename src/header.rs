use crate::pmt::{Tag, Timestamp};
use num_complex::Complex;
use std::{any::TypeId, marker::PhantomData, rc::Rc};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum InvalidHeaderError {
    #[error("Header was not a dictionary")]
    HeaderNotDictionary,
    #[error("Missing field {0} in header")]
    MissingField(&'static str),
    #[error("Field {0} was present in header, but was of unexpected type")]
    WrongTypeField(&'static str),
    #[error("Type {0} was present in header, but this represents no known data type")]
    WrongDataType(i32),
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
    pub fn is_floating(&self) -> bool {
        return *self == DataType::Float || *self == DataType::Double;
    }

    /// Only returns true if the type is directly representable as the target type, including signed-ness
    /// and number of bits of the type.
    pub fn reads_directly_to<T: 'static>(&self, complex: bool) -> bool {
        if complex {
            match *self {
                DataType::Byte => TypeId::of::<T>() == TypeId::of::<Complex<i8>>(),
                DataType::Short => TypeId::of::<T>() == TypeId::of::<Complex<i16>>(),
                DataType::Int => TypeId::of::<T>() == TypeId::of::<Complex<i32>>(),
                DataType::Float => TypeId::of::<T>() == TypeId::of::<Complex<f32>>(),
                DataType::Double => TypeId::of::<T>() == TypeId::of::<Complex<f64>>(),
            }
        } else {
            match *self {
                DataType::Byte => TypeId::of::<T>() == TypeId::of::<i8>(),
                DataType::Short => TypeId::of::<T>() == TypeId::of::<i16>(),
                DataType::Int => TypeId::of::<T>() == TypeId::of::<i32>(),
                DataType::Float => TypeId::of::<T>() == TypeId::of::<f32>(),
                DataType::Double => TypeId::of::<T>() == TypeId::of::<f64>(),
            }
        }
    }

    /// No edge cases, we only support up-casting among the basic types from GNU Radio, and
    /// floating point conversion (lossy conversion from f64 -> f32 is allowed!)
    pub fn converts_to<T: 'static>(&self, complex: bool) -> bool {
        if complex {
            match *self {
                DataType::Byte => {
                    TypeId::of::<T>() == TypeId::of::<Complex<f64>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<f32>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<i32>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<i16>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<i8>>()
                }
                DataType::Short => {
                    TypeId::of::<T>() == TypeId::of::<Complex<f64>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<f32>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<i32>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<i16>>()
                }
                DataType::Int => {
                    TypeId::of::<T>() == TypeId::of::<Complex<f64>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<f32>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<i32>>()
                }
                DataType::Float => {
                    TypeId::of::<T>() == TypeId::of::<Complex<f64>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<f32>>()
                }
                DataType::Double => {
                    TypeId::of::<T>() == TypeId::of::<Complex<f64>>()
                        || TypeId::of::<T>() == TypeId::of::<Complex<f32>>()
                }
            }
        } else {
            match *self {
                DataType::Byte => {
                    TypeId::of::<T>() == TypeId::of::<f64>()
                        || TypeId::of::<T>() == TypeId::of::<f32>()
                        || TypeId::of::<T>() == TypeId::of::<i32>()
                        || TypeId::of::<T>() == TypeId::of::<i16>()
                        || TypeId::of::<T>() == TypeId::of::<i8>()
                }
                DataType::Short => {
                    TypeId::of::<T>() == TypeId::of::<f64>()
                        || TypeId::of::<T>() == TypeId::of::<f32>()
                        || TypeId::of::<T>() == TypeId::of::<i32>()
                        || TypeId::of::<T>() == TypeId::of::<i16>()
                }
                DataType::Int => {
                    TypeId::of::<T>() == TypeId::of::<f64>()
                        || TypeId::of::<T>() == TypeId::of::<f32>()
                        || TypeId::of::<T>() == TypeId::of::<i32>()
                }
                DataType::Float => {
                    TypeId::of::<T>() == TypeId::of::<f64>()
                        || TypeId::of::<T>() == TypeId::of::<f32>()
                }
                DataType::Double => {
                    TypeId::of::<T>() == TypeId::of::<f64>()
                        || TypeId::of::<T>() == TypeId::of::<f32>()
                }
            }
        }
    }

    pub fn converts_to_dtype(&self, other: &Self) -> bool {
        todo!("Implement");
    }

    pub fn read_from_bytes<T>(&self, bytes: &[u8]) -> T {
        todo!("Implement");
    }

    pub fn from_int(i: i32) -> Result<Self, InvalidHeaderError> {
        Ok(match i {
            0 => Self::Byte,
            _ => return Err(InvalidHeaderError::WrongDataType(i)),
        })
    }
}

/// Header as read from the GNU radio file
#[derive(PartialEq, Debug, Clone)]
pub struct Header {
    // TODO: Make fields private and use getters
    /// Sample rate of the data
    pub samp_rate: f64,
    /// Duration of a sample, computed from samp_rate
    pub samp_dur: f64,
    /// Reception time of the first sample of the data, relative to first sample
    pub rx_time: Timestamp,
    /// Size of the item in bytes
    pub size: i32,
    /// Type of the data
    pub dtype: DataType,
    /// Is the data complex?
    pub cplx: bool,
    /// Offset to the first byte of data in this header's segment
    pub strt: u64,
    /// Size in bytes of the data in this header's segment
    pub bytes: u64,

    pub extra_dict: Rc<Tag>,

    /// Absolute position of the first byte of the data from the start of the file,
    /// computed by ourselves
    pub abs_pos: u64,

    /// Absolute position of the first byte of the HEADER in the file (either attached or dettached),
    /// computed by ourselves
    pub pos_in_file: u64,
}

impl Header {
    pub fn get_num_samples(&self) -> u64 {
        todo!("Implement");
    }

    /// Returns the expected reception time of sample at offset `sample` (which
    /// may be outside the header just fine, or even negative) assuming the sample rate is held
    /// constant until said offset.
    pub fn get_sample_time(&self, sample: i64) -> Timestamp {
        todo!("Implement");
    }

    /// Gets the duration of a sample at the sample rate of the header
    pub fn get_sample_duration(&self) -> f64 {
        self.samp_dur
    }

    pub fn is_compatible_with(&self, other: &Header, preserve: SeekPreserve) -> bool {
        if preserve.preserves_samplerate() && other.samp_rate != self.samp_rate {
            return false;
        }
        if preserve.preserves_format() && other.dtype != self.dtype {
            return false;
        }
        if preserve.preserves_convertability() && !other.dtype.converts_to_dtype(&self.dtype) {
            return false;
        }

        true
    }

    /// Returns true if the first sample of this header is received at most with a time error
    /// of 0.1 * other.get_sample_duration(), to account for floating point errors.
    pub fn is_continuation_of(&self, other: &Header) -> bool {
        let last_sample_t = if other.get_num_samples() == 0 {
            other.rx_time
        } else {
            other.get_sample_time(other.get_num_samples() as i64 - 1)
        };
        let diff = self.rx_time.abs_diff(last_sample_t).to_num::<f64>();
        diff <= 0.1 * other.get_sample_duration()
    }

    pub fn get_sample_pos_of_byte(&self, byte: u64) -> u64 {
        todo!("Implement");
    }

    pub fn from_tags(
        byte_in_file: u64,
        tag: Tag,
        extra: Tag,
    ) -> Result<Header, InvalidHeaderError> {
        let tag = if let Tag::Dict(as_dict) = tag {
            as_dict
        } else {
            return Err(InvalidHeaderError::HeaderNotDictionary);
        };

        println!("Read header from tag {:?}", tag);
        println!("Extra: {:?}", extra);

        let samp_rate = tag
            .get("rx_rate")
            .ok_or(InvalidHeaderError::MissingField("rx_rate"))?
            .get_f64()
            .ok_or(InvalidHeaderError::WrongTypeField("rx_rate"))?;

        let samp_dur = 1.0 / samp_rate;

        let (rx_time_a, rx_time_b) = match tag
            .get("rx_time")
            .ok_or(InvalidHeaderError::MissingField("rx_time"))?
        {
            Tag::Tuple(vec) => {
                let a = vec
                    .get(0)
                    .ok_or(InvalidHeaderError::MissingField("rx_time seconds"))?;
                let b = vec
                    .get(1)
                    .ok_or(InvalidHeaderError::MissingField("rx_time fractional"))?;
                (a, b)
            }
            _ => return Err(InvalidHeaderError::WrongTypeField("rx_time")),
        };

        let rx_time_secs = rx_time_a
            .get_u64()
            .ok_or(InvalidHeaderError::WrongTypeField("rx_time seconds"))?;
        let rx_time_frac = rx_time_b
            .get_f64()
            .ok_or(InvalidHeaderError::WrongTypeField("rx_time fraction"))?;

        let rx_time = Timestamp::from_num(rx_time_secs) + Timestamp::from_num(rx_time_frac);

        let size = tag
            .get("size")
            .ok_or(InvalidHeaderError::MissingField("size"))?
            .get_i32()
            .ok_or(InvalidHeaderError::WrongTypeField("size"))?;

        let dtype = DataType::from_int(
            tag.get("type")
                .ok_or(InvalidHeaderError::MissingField("type"))?
                .get_i32()
                .ok_or(InvalidHeaderError::WrongTypeField("type"))?,
        )?;

        let cplx = tag
            .get("cplx")
            .ok_or(InvalidHeaderError::MissingField("cplx"))?
            .get_bool()
            .ok_or(InvalidHeaderError::WrongTypeField("cplx"))?;

        let strt = tag
            .get("strt")
            .ok_or(InvalidHeaderError::MissingField("strt"))?
            .get_u64()
            .ok_or(InvalidHeaderError::WrongTypeField("strt"))?;

        let bytes = tag
            .get("bytes")
            .ok_or(InvalidHeaderError::MissingField("bytes"))?
            .get_u64()
            .ok_or(InvalidHeaderError::WrongTypeField("bytes"))?;

        Ok(Header {
            samp_rate,
            samp_dur,
            rx_time,
            size,
            dtype,
            cplx,
            strt,
            bytes,
            extra_dict: Rc::new(extra),
            // TODO: THESE ARE INCORRECT
            abs_pos: strt,
            pos_in_file: byte_in_file,
        })
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;

    // Some very tedious tests ahead...
    #[test]
    fn dtype_byte() {
        assert!(!DataType::Byte.is_floating());

        assert!(DataType::Byte.reads_directly_to::<i8>(false));
        assert!(!DataType::Byte.reads_directly_to::<u8>(false));
        assert!(!DataType::Byte.reads_directly_to::<i16>(false));
        assert!(!DataType::Byte.reads_directly_to::<u16>(false));
        assert!(!DataType::Byte.reads_directly_to::<i32>(false));
        assert!(!DataType::Byte.reads_directly_to::<u32>(false));
        assert!(!DataType::Byte.reads_directly_to::<i64>(false));
        assert!(!DataType::Byte.reads_directly_to::<u64>(false));
        assert!(!DataType::Byte.reads_directly_to::<f32>(false));
        assert!(!DataType::Byte.reads_directly_to::<f64>(false));

        assert!(DataType::Byte.converts_to::<i8>(false));
        assert!(!DataType::Byte.converts_to::<u8>(false));
        assert!(DataType::Byte.converts_to::<i16>(false));
        assert!(!DataType::Byte.converts_to::<u16>(false));
        assert!(DataType::Byte.converts_to::<i32>(false));
        assert!(!DataType::Byte.converts_to::<u32>(false));
        assert!(!DataType::Byte.converts_to::<i64>(false));
        assert!(!DataType::Byte.converts_to::<u64>(false));
        assert!(DataType::Byte.converts_to::<f32>(false));
        assert!(DataType::Byte.converts_to::<f64>(false));

        assert!(DataType::Byte.reads_directly_to::<Complex<i8>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<u8>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<i16>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<u16>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<i32>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<u32>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<i64>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<u64>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<f32>>(true));
        assert!(!DataType::Byte.reads_directly_to::<Complex<f64>>(true));

        assert!(DataType::Byte.converts_to::<Complex<i8>>(true));
        assert!(!DataType::Byte.converts_to::<Complex<u8>>(true));
        assert!(DataType::Byte.converts_to::<Complex<i16>>(true));
        assert!(!DataType::Byte.converts_to::<Complex<u16>>(true));
        assert!(DataType::Byte.converts_to::<Complex<i32>>(true));
        assert!(!DataType::Byte.converts_to::<Complex<u32>>(true));
        assert!(!DataType::Byte.converts_to::<Complex<i64>>(true));
        assert!(!DataType::Byte.converts_to::<Complex<u64>>(true));
        assert!(DataType::Byte.converts_to::<Complex<f32>>(true));
        assert!(DataType::Byte.converts_to::<Complex<f64>>(true));

        assert!(!DataType::Byte.reads_directly_to::<Complex<i8>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<u8>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<i16>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<u16>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<i32>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<u32>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<i64>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<u64>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<f32>>(false));
        assert!(!DataType::Byte.reads_directly_to::<Complex<f64>>(false));

        assert!(!DataType::Byte.reads_directly_to::<i8>(true));
        assert!(!DataType::Byte.reads_directly_to::<u8>(true));
        assert!(!DataType::Byte.reads_directly_to::<i16>(true));
        assert!(!DataType::Byte.reads_directly_to::<u16>(true));
        assert!(!DataType::Byte.reads_directly_to::<i32>(true));
        assert!(!DataType::Byte.reads_directly_to::<u32>(true));
        assert!(!DataType::Byte.reads_directly_to::<i64>(true));
        assert!(!DataType::Byte.reads_directly_to::<u64>(true));
        assert!(!DataType::Byte.reads_directly_to::<f32>(true));
        assert!(!DataType::Byte.reads_directly_to::<f64>(true));
    }

    #[test]
    fn dtype_short() {
        assert!(!DataType::Short.is_floating());

        assert!(!DataType::Short.reads_directly_to::<i8>(false));
        assert!(!DataType::Short.reads_directly_to::<u8>(false));
        assert!(DataType::Short.reads_directly_to::<i16>(false));
        assert!(!DataType::Short.reads_directly_to::<u16>(false));
        assert!(!DataType::Short.reads_directly_to::<i32>(false));
        assert!(!DataType::Short.reads_directly_to::<u32>(false));
        assert!(!DataType::Short.reads_directly_to::<i64>(false));
        assert!(!DataType::Short.reads_directly_to::<u64>(false));
        assert!(!DataType::Short.reads_directly_to::<f32>(false));
        assert!(!DataType::Short.reads_directly_to::<f64>(false));

        assert!(!DataType::Short.converts_to::<i8>(false));
        assert!(!DataType::Short.converts_to::<u8>(false));
        assert!(DataType::Short.converts_to::<i16>(false));
        assert!(!DataType::Short.converts_to::<u16>(false));
        assert!(DataType::Short.converts_to::<i32>(false));
        assert!(!DataType::Short.converts_to::<u32>(false));
        assert!(!DataType::Short.converts_to::<i64>(false));
        assert!(!DataType::Short.converts_to::<u64>(false));
        assert!(DataType::Short.converts_to::<f32>(false));
        assert!(DataType::Short.converts_to::<f64>(false));

        assert!(!DataType::Short.reads_directly_to::<Complex<i8>>(true));
        assert!(!DataType::Short.reads_directly_to::<Complex<u8>>(true));
        assert!(DataType::Short.reads_directly_to::<Complex<i16>>(true));
        assert!(!DataType::Short.reads_directly_to::<Complex<u16>>(true));
        assert!(!DataType::Short.reads_directly_to::<Complex<i32>>(true));
        assert!(!DataType::Short.reads_directly_to::<Complex<u32>>(true));
        assert!(!DataType::Short.reads_directly_to::<Complex<i64>>(true));
        assert!(!DataType::Short.reads_directly_to::<Complex<u64>>(true));
        assert!(!DataType::Short.reads_directly_to::<Complex<f32>>(true));
        assert!(!DataType::Short.reads_directly_to::<Complex<f64>>(true));

        assert!(!DataType::Short.converts_to::<Complex<i8>>(true));
        assert!(!DataType::Short.converts_to::<Complex<u8>>(true));
        assert!(DataType::Short.converts_to::<Complex<i16>>(true));
        assert!(!DataType::Short.converts_to::<Complex<u16>>(true));
        assert!(DataType::Short.converts_to::<Complex<i32>>(true));
        assert!(!DataType::Short.converts_to::<Complex<u32>>(true));
        assert!(!DataType::Short.converts_to::<Complex<i64>>(true));
        assert!(!DataType::Short.converts_to::<Complex<u64>>(true));
        assert!(DataType::Short.converts_to::<Complex<f32>>(true));
        assert!(DataType::Short.converts_to::<Complex<f64>>(true));

        assert!(!DataType::Short.reads_directly_to::<Complex<i8>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<u8>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<i16>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<u16>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<i32>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<u32>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<i64>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<u64>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<f32>>(false));
        assert!(!DataType::Short.reads_directly_to::<Complex<f64>>(false));

        assert!(!DataType::Short.reads_directly_to::<i8>(true));
        assert!(!DataType::Short.reads_directly_to::<u8>(true));
        assert!(!DataType::Short.reads_directly_to::<i16>(true));
        assert!(!DataType::Short.reads_directly_to::<u16>(true));
        assert!(!DataType::Short.reads_directly_to::<i32>(true));
        assert!(!DataType::Short.reads_directly_to::<u32>(true));
        assert!(!DataType::Short.reads_directly_to::<i64>(true));
        assert!(!DataType::Short.reads_directly_to::<u64>(true));
        assert!(!DataType::Short.reads_directly_to::<f32>(true));
        assert!(!DataType::Short.reads_directly_to::<f64>(true));
    }

    #[test]
    fn dtype_int() {
        assert!(!DataType::Int.is_floating());

        assert!(!DataType::Int.reads_directly_to::<i8>(false));
        assert!(!DataType::Int.reads_directly_to::<u8>(false));
        assert!(!DataType::Int.reads_directly_to::<i16>(false));
        assert!(!DataType::Int.reads_directly_to::<u16>(false));
        assert!(DataType::Int.reads_directly_to::<i32>(false));
        assert!(!DataType::Int.reads_directly_to::<u32>(false));
        assert!(!DataType::Int.reads_directly_to::<i64>(false));
        assert!(!DataType::Int.reads_directly_to::<u64>(false));
        assert!(!DataType::Int.reads_directly_to::<f32>(false));
        assert!(!DataType::Int.reads_directly_to::<f64>(false));

        assert!(!DataType::Int.converts_to::<i8>(false));
        assert!(!DataType::Int.converts_to::<u8>(false));
        assert!(!DataType::Int.converts_to::<i16>(false));
        assert!(!DataType::Int.converts_to::<u16>(false));
        assert!(DataType::Int.converts_to::<i32>(false));
        assert!(!DataType::Int.converts_to::<u32>(false));
        assert!(!DataType::Int.converts_to::<i64>(false));
        assert!(!DataType::Int.converts_to::<u64>(false));
        assert!(DataType::Int.converts_to::<f32>(false));
        assert!(DataType::Int.converts_to::<f64>(false));

        assert!(!DataType::Int.reads_directly_to::<Complex<i8>>(true));
        assert!(!DataType::Int.reads_directly_to::<Complex<u8>>(true));
        assert!(!DataType::Int.reads_directly_to::<Complex<i16>>(true));
        assert!(!DataType::Int.reads_directly_to::<Complex<u16>>(true));
        assert!(DataType::Int.reads_directly_to::<Complex<i32>>(true));
        assert!(!DataType::Int.reads_directly_to::<Complex<u32>>(true));
        assert!(!DataType::Int.reads_directly_to::<Complex<i64>>(true));
        assert!(!DataType::Int.reads_directly_to::<Complex<u64>>(true));
        assert!(!DataType::Int.reads_directly_to::<Complex<f32>>(true));
        assert!(!DataType::Int.reads_directly_to::<Complex<f64>>(true));

        assert!(!DataType::Int.converts_to::<Complex<i8>>(true));
        assert!(!DataType::Int.converts_to::<Complex<u8>>(true));
        assert!(!DataType::Int.converts_to::<Complex<i16>>(true));
        assert!(!DataType::Int.converts_to::<Complex<u16>>(true));
        assert!(DataType::Int.converts_to::<Complex<i32>>(true));
        assert!(!DataType::Int.converts_to::<Complex<u32>>(true));
        assert!(!DataType::Int.converts_to::<Complex<i64>>(true));
        assert!(!DataType::Int.converts_to::<Complex<u64>>(true));
        assert!(DataType::Int.converts_to::<Complex<f32>>(true));
        assert!(DataType::Int.converts_to::<Complex<f64>>(true));

        assert!(!DataType::Int.reads_directly_to::<Complex<i8>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<u8>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<i16>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<u16>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<i32>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<u32>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<i64>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<u64>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<f32>>(false));
        assert!(!DataType::Int.reads_directly_to::<Complex<f64>>(false));

        assert!(!DataType::Int.reads_directly_to::<i8>(true));
        assert!(!DataType::Int.reads_directly_to::<u8>(true));
        assert!(!DataType::Int.reads_directly_to::<i16>(true));
        assert!(!DataType::Int.reads_directly_to::<u16>(true));
        assert!(!DataType::Int.reads_directly_to::<i32>(true));
        assert!(!DataType::Int.reads_directly_to::<u32>(true));
        assert!(!DataType::Int.reads_directly_to::<i64>(true));
        assert!(!DataType::Int.reads_directly_to::<u64>(true));
        assert!(!DataType::Int.reads_directly_to::<f32>(true));
        assert!(!DataType::Int.reads_directly_to::<f64>(true));
    }

    #[test]
    fn dtype_float() {
        assert!(DataType::Float.is_floating());

        assert!(!DataType::Float.reads_directly_to::<i8>(false));
        assert!(!DataType::Float.reads_directly_to::<u8>(false));
        assert!(!DataType::Float.reads_directly_to::<i16>(false));
        assert!(!DataType::Float.reads_directly_to::<u16>(false));
        assert!(!DataType::Float.reads_directly_to::<i32>(false));
        assert!(!DataType::Float.reads_directly_to::<u32>(false));
        assert!(!DataType::Float.reads_directly_to::<i64>(false));
        assert!(!DataType::Float.reads_directly_to::<u64>(false));
        assert!(DataType::Float.reads_directly_to::<f32>(false));
        assert!(!DataType::Float.reads_directly_to::<f64>(false));

        assert!(!DataType::Float.converts_to::<i8>(false));
        assert!(!DataType::Float.converts_to::<u8>(false));
        assert!(!DataType::Float.converts_to::<i16>(false));
        assert!(!DataType::Float.converts_to::<u16>(false));
        assert!(!DataType::Float.converts_to::<i32>(false));
        assert!(!DataType::Float.converts_to::<u32>(false));
        assert!(!DataType::Float.converts_to::<i64>(false));
        assert!(!DataType::Float.converts_to::<u64>(false));
        assert!(DataType::Float.converts_to::<f32>(false));
        assert!(DataType::Float.converts_to::<f64>(false));

        assert!(!DataType::Float.reads_directly_to::<Complex<i8>>(true));
        assert!(!DataType::Float.reads_directly_to::<Complex<u8>>(true));
        assert!(!DataType::Float.reads_directly_to::<Complex<i16>>(true));
        assert!(!DataType::Float.reads_directly_to::<Complex<u16>>(true));
        assert!(!DataType::Float.reads_directly_to::<Complex<i32>>(true));
        assert!(!DataType::Float.reads_directly_to::<Complex<u32>>(true));
        assert!(!DataType::Float.reads_directly_to::<Complex<i64>>(true));
        assert!(!DataType::Float.reads_directly_to::<Complex<u64>>(true));
        assert!(DataType::Float.reads_directly_to::<Complex<f32>>(true));
        assert!(!DataType::Float.reads_directly_to::<Complex<f64>>(true));

        assert!(!DataType::Float.converts_to::<Complex<i8>>(true));
        assert!(!DataType::Float.converts_to::<Complex<u8>>(true));
        assert!(!DataType::Float.converts_to::<Complex<i16>>(true));
        assert!(!DataType::Float.converts_to::<Complex<u16>>(true));
        assert!(!DataType::Float.converts_to::<Complex<i32>>(true));
        assert!(!DataType::Float.converts_to::<Complex<u32>>(true));
        assert!(!DataType::Float.converts_to::<Complex<i64>>(true));
        assert!(!DataType::Float.converts_to::<Complex<u64>>(true));
        assert!(DataType::Float.converts_to::<Complex<f32>>(true));
        assert!(DataType::Float.converts_to::<Complex<f64>>(true));

        assert!(!DataType::Float.reads_directly_to::<Complex<i8>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<u8>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<i16>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<u16>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<i32>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<u32>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<i64>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<u64>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<f32>>(false));
        assert!(!DataType::Float.reads_directly_to::<Complex<f64>>(false));

        assert!(!DataType::Float.reads_directly_to::<i8>(true));
        assert!(!DataType::Float.reads_directly_to::<u8>(true));
        assert!(!DataType::Float.reads_directly_to::<i16>(true));
        assert!(!DataType::Float.reads_directly_to::<u16>(true));
        assert!(!DataType::Float.reads_directly_to::<i32>(true));
        assert!(!DataType::Float.reads_directly_to::<u32>(true));
        assert!(!DataType::Float.reads_directly_to::<i64>(true));
        assert!(!DataType::Float.reads_directly_to::<u64>(true));
        assert!(!DataType::Float.reads_directly_to::<f32>(true));
        assert!(!DataType::Float.reads_directly_to::<f64>(true));
    }

    #[test]
    fn dtype_double() {
        assert!(DataType::Double.is_floating());

        assert!(!DataType::Double.reads_directly_to::<i8>(false));
        assert!(!DataType::Double.reads_directly_to::<u8>(false));
        assert!(!DataType::Double.reads_directly_to::<i16>(false));
        assert!(!DataType::Double.reads_directly_to::<u16>(false));
        assert!(!DataType::Double.reads_directly_to::<i32>(false));
        assert!(!DataType::Double.reads_directly_to::<u32>(false));
        assert!(!DataType::Double.reads_directly_to::<i64>(false));
        assert!(!DataType::Double.reads_directly_to::<u64>(false));
        assert!(!DataType::Double.reads_directly_to::<f32>(false));
        assert!(DataType::Double.reads_directly_to::<f64>(false));

        assert!(!DataType::Double.converts_to::<i8>(false));
        assert!(!DataType::Double.converts_to::<u8>(false));
        assert!(!DataType::Double.converts_to::<i16>(false));
        assert!(!DataType::Double.converts_to::<u16>(false));
        assert!(!DataType::Double.converts_to::<i32>(false));
        assert!(!DataType::Double.converts_to::<u32>(false));
        assert!(!DataType::Double.converts_to::<i64>(false));
        assert!(!DataType::Double.converts_to::<u64>(false));
        assert!(DataType::Double.converts_to::<f32>(false));
        assert!(DataType::Double.converts_to::<f64>(false));

        assert!(!DataType::Double.reads_directly_to::<Complex<i8>>(true));
        assert!(!DataType::Double.reads_directly_to::<Complex<u8>>(true));
        assert!(!DataType::Double.reads_directly_to::<Complex<i16>>(true));
        assert!(!DataType::Double.reads_directly_to::<Complex<u16>>(true));
        assert!(!DataType::Double.reads_directly_to::<Complex<i32>>(true));
        assert!(!DataType::Double.reads_directly_to::<Complex<u32>>(true));
        assert!(!DataType::Double.reads_directly_to::<Complex<i64>>(true));
        assert!(!DataType::Double.reads_directly_to::<Complex<u64>>(true));
        assert!(!DataType::Double.reads_directly_to::<Complex<f32>>(true));
        assert!(DataType::Double.reads_directly_to::<Complex<f64>>(true));

        assert!(!DataType::Double.converts_to::<Complex<i8>>(true));
        assert!(!DataType::Double.converts_to::<Complex<u8>>(true));
        assert!(!DataType::Double.converts_to::<Complex<i16>>(true));
        assert!(!DataType::Double.converts_to::<Complex<u16>>(true));
        assert!(!DataType::Double.converts_to::<Complex<i32>>(true));
        assert!(!DataType::Double.converts_to::<Complex<u32>>(true));
        assert!(!DataType::Double.converts_to::<Complex<i64>>(true));
        assert!(!DataType::Double.converts_to::<Complex<u64>>(true));
        assert!(DataType::Double.converts_to::<Complex<f32>>(true));
        assert!(DataType::Double.converts_to::<Complex<f64>>(true));

        assert!(!DataType::Double.reads_directly_to::<Complex<i8>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<u8>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<i16>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<u16>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<i32>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<u32>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<i64>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<u64>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<f32>>(false));
        assert!(!DataType::Double.reads_directly_to::<Complex<f64>>(false));

        assert!(!DataType::Double.reads_directly_to::<i8>(true));
        assert!(!DataType::Double.reads_directly_to::<u8>(true));
        assert!(!DataType::Double.reads_directly_to::<i16>(true));
        assert!(!DataType::Double.reads_directly_to::<u16>(true));
        assert!(!DataType::Double.reads_directly_to::<i32>(true));
        assert!(!DataType::Double.reads_directly_to::<u32>(true));
        assert!(!DataType::Double.reads_directly_to::<i64>(true));
        assert!(!DataType::Double.reads_directly_to::<u64>(true));
        assert!(!DataType::Double.reads_directly_to::<f32>(true));
        assert!(!DataType::Double.reads_directly_to::<f64>(true));
    }
}
