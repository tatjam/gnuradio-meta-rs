//! A most basic PMT parser. It's compatible with the format used as of GNU Radio version 3.10.9.2.
//! We only support the bare basics to parse the meta headers, see the imhex pattern file in the repo.

use byteorder::{BigEndian, ReadBytesExt};
use std::{collections::HashMap, io::Read};
use thiserror::Error;

type StringToTag = HashMap<String, Tag>;

#[derive(PartialEq, Debug)]
pub enum Tag {
    Bool(bool),
    Symbol(String),
    Int32(i32),
    Double(f64),
    Null(),
    Pair(Box<Tag>, Box<Tag>),
    Dict(StringToTag),
    UInt64(u64),
    Tuple(Vec<Tag>),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected EOF while parsing")]
    UnexpectedEOF(),
    #[error("Dict entry didn't follow dict(pair(name_a, a), ...) structure")]
    MalformedDict(),
    #[error("Reader I/O error while parsing")]
    IoError(#[from] std::io::Error),
    #[error("Symbol was not UTF-8 encoded, likely corrupt file")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

fn parse_symbol<T: Read>(reader: &mut T) -> Result<Tag, ParseError> {
    let len = reader.read_u16::<BigEndian>()?;
    let mut bytes = Vec::with_capacity(len as usize);

    let bytes_read = reader.read(bytes.as_mut_slice())?;
    if bytes_read != len as usize {
        return Err(ParseError::UnexpectedEOF());
    }

    Ok(Tag::Symbol(String::from_utf8(bytes)?))
}

fn parse_pair_inner<T: Read>(reader: &mut T) -> Result<(Tag, Tag), ParseError> {
    let first = parse(reader)?;
    let second = parse(reader)?;
    Ok((first, second))
}

fn parse_pair<T: Read>(reader: &mut T) -> Result<Tag, ParseError> {
    let ab = parse_pair_inner(reader)?;
    Ok(Tag::Pair(Box::new(ab.0), Box::new(ab.1)))
}

fn expect_byte<T: Read>(reader: &mut T) -> Result<u8, ParseError> {
    let mut byte_buf: [u8; 1] = Default::default();
    let num_read = reader.read(&mut byte_buf)?;
    if num_read != 1 {
        // EOF or similar
        return Err(ParseError::UnexpectedEOF());
    }

    Ok(byte_buf[0])
}

fn parse_dict_inner<T: Read>(rdr: &mut T, tgt: &mut StringToTag) -> Result<(), ParseError> {
    // The "pair" byte
    if expect_byte(rdr)? != 0x7 {
        return Err(ParseError::MalformedDict());
    }

    let pair = parse_pair_inner(rdr)?;

    if let Tag::Symbol(name) = pair.0 {
        tgt.insert(name, pair.1);
    } else {
        return Err(ParseError::MalformedDict());
    }

    let next_byte = expect_byte(rdr)?;

    match next_byte {
        0x6 => Ok(()),                     // null byte, dict is over
        0x9 => parse_dict_inner(rdr, tgt), // dict byte, continue parsing
        _ => Err(ParseError::MalformedDict()),
    }
}

fn parse_dict<T: Read>(reader: &mut T) -> Result<Tag, ParseError> {
    // A dict is formed as dict(pair(name_a, a), dict(pair(name_b, b), ...))
    let mut dict = HashMap::new();
    parse_dict_inner(reader, &mut dict)?;
    Ok(Tag::Dict(dict))
}

fn parse_tuple<T: Read>(reader: &mut T) -> Result<Tag, ParseError> {
    let num = reader.read_u32::<BigEndian>()?;
    let mut vec = Vec::with_capacity(num as usize);
    for _ in 0..num {
        vec.push(parse(reader)?)
    }
    Ok(Tag::Tuple(vec))
}

fn parse_tag<T: Read>(reader: &mut T, kind: u8) -> Result<Tag, ParseError> {
    match kind {
        0x0 => Ok(Tag::Bool(true)),
        0x1 => Ok(Tag::Bool(false)),
        0x2 => parse_symbol(reader),
        0x3 => Ok(Tag::Int32(reader.read_i32::<BigEndian>()?)),
        0x4 => Ok(Tag::Double(reader.read_f64::<BigEndian>()?)),
        0x6 => Ok(Tag::Null()),
        0x7 => parse_pair(reader),
        0x9 => parse_dict(reader),
        0xb => Ok(Tag::UInt64(reader.read_u64::<BigEndian>()?)),
        0xc => parse_tuple(reader),
        _x => todo!("Unimplemented"),
    }
}

/// The reader must be positioned at the start of a Tag
pub fn parse<T: Read>(reader: &mut T) -> Result<Tag, ParseError> {
    let byte = expect_byte(reader)?;
    parse_tag(reader, byte)
}

/// Tries to read a tag, but if EOF is found on the first read, None is returned
/// instead of an error.
/// The reader must be positioned at the start of a Tag
pub fn parse_maybe_eof<T: Read>(reader: &mut T) -> Result<Option<Tag>, ParseError> {
    let byte = match expect_byte(reader) {
        Err(e) => match e {
            ParseError::UnexpectedEOF() => return Ok(None),
            _ => return Err(e),
        },
        Ok(v) => v,
    };
    match parse_tag(reader, byte) {
        Err(e) => Err(e),
        Ok(v) => Ok(Some(v)),
    }
}
