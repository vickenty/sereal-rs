use std::io;
use std::result;

use byteorder::{ LittleEndian, ByteOrder };
use sereal_common::constants::*;

use varint;
use varint::VarintReaderExt;

use config::Config;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    UnexpectedEof,
    VarintOverflow,
    StringTooLarge(u64),
    UnknownTag(u8),
}

impl Error {
    pub fn is_eof(&self) -> bool {
        match *self {
            Error::UnexpectedEof => true,
            Error::IOError(ref e) => e.kind() == io::ErrorKind::UnexpectedEof,
            _ => false,
        }
    }

    pub fn is_varint_overflow(&self) -> bool {
        match *self {
            Error::VarintOverflow => true,
            _ => false
        }
    }

    pub fn is_unknown_tag(&self) -> bool {
        self.as_unknown_tag().is_some()
    }

    pub fn as_unknown_tag(&self) -> Option<u8> {
        match *self {
            Error::UnknownTag(t) => Some(t),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IOError(e)
    }
}

impl From<varint::Error> for Error {
    fn from(e: varint::Error) -> Error {
        match e {
            varint::Error::Overflow => Error::VarintOverflow,
            varint::Error::IOError(e) => Error::IOError(e),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Tag<'a> {
    Undef,
    CanonicalUndef,
    True,
    False,
    Pos(u8),
    Neg(i8),
    Varint(u64),
    Zigzag(i64),
    Float(f32),
    Double(f64),
    Refn,
    Refp(u64),
    Alias(u64),
    Copy(u64),
    Array(u64),
    ArrayRef(u8),
    Hash(u64),
    HashRef(u8),
    Bin(&'a [u8]),
    Str(&'a [u8]),
    Object,
    ObjectV(u64),
    ObjectFreeze,
    ObjectVFreeze(u64),
    Weaken,
    Regexp,
}

pub struct Token<'a> {
    pub pos: u64,
    pub track: bool,
    pub tag: Tag<'a>,
}

pub struct Lexer<'a, 'b> {
    config: &'a Config,
    input: &'b [u8],
    pos: usize,
}

impl<'a, 'b> Lexer<'a, 'b> {
    pub fn next(&mut self) -> Result<Token<'b>> {
        let tag = self.read_tag()?;
        let pos = self.pos as u64;
        let trk = tag & TRACK_BIT == TRACK_BIT;
        let tag = tag & TYPE_MASK;

        let value = match tag {
            POS_0...
            POS_15 => Tag::Pos(tag),

            NEG_16...
            NEG_1 => Tag::Neg((tag | 0xf0) as i8),

            VARINT => Tag::Varint(self.read_varint()?),
            ZIGZAG => Tag::Zigzag(self.read_zigzag()?),

            FLOAT => Tag::Float(self.read_f32()?),
            DOUBLE => Tag::Double(self.read_f64()?),
            LONG_DOUBLE => return Err(Error::UnknownTag(tag)),

            UNDEF => Tag::Undef,

            BINARY => Tag::Bin(self.read_varint().and_then(|l| Ok(self.read_bytes(l)?))?),
            STR_UTF8 => Tag::Str(self.read_varint().and_then(|l| Ok(self.read_bytes(l)?))?),

            REFN => Tag::Refn,
            REFP => Tag::Refp(self.read_varint()?),

            ARRAY => Tag::Array(self.read_varint()?),
            HASH => Tag::Hash(self.read_varint()?),

            OBJECT => Tag::Object,
            OBJECTV => Tag::ObjectV(self.read_varint()?),

            ALIAS => Tag::Alias(self.read_varint()?),
            COPY => Tag::Copy(self.read_varint()?),
            WEAKEN => Tag::Weaken,
            REGEXP => Tag::Regexp,

            OBJECT_FREEZE => Tag::ObjectFreeze,
            OBJECTV_FREEZE => Tag::ObjectVFreeze(self.read_varint()?),

            RESERVED_0...
            RESERVED_4 => return Err(Error::UnknownTag(tag)),

            CANONICAL_UNDEF => Tag::CanonicalUndef,
            FALSE => Tag::False,
            TRUE => Tag::True,

            MANY => return Err(Error::UnknownTag(tag)),
            PACKET_START => return Err(Error::UnknownTag(tag)),
            EXTEND => return Err(Error::UnknownTag(tag)),

            PAD => panic!("PAD should be handled in read_tag()"),

            ARRAYREF_0...
            ARRAYREF_15 => Tag::ArrayRef(tag - ARRAYREF_0),

            HASHREF_0...
            HASHREF_15 => Tag::HashRef(tag - HASHREF_0),

            SHORT_BINARY_0 => Tag::Bin(&[]),
            SHORT_BINARY_1...
            SHORT_BINARY_31 => Tag::Bin(self.read_bytes((tag - SHORT_BINARY_0) as u64)?),

            _ => return Err(Error::UnknownTag(tag)),
        };

        Ok(Token {
            pos: pos,
            track: trk,
            tag: value,
        })
    }

    pub fn new(config: &'a Config, input: &'b [u8]) -> Lexer<'a, 'b> {
        Lexer {
            config: config,
            input: input,
            pos: 0,
        }
    }

    fn next_byte(&mut self) -> Result<u8> {
        if self.pos < self.input.len() {
            let b = self.input[self.pos];
            self.pos += 1;
            Ok(b)
        } else {
            Err(Error::UnexpectedEof)
        }
    }

    fn read_tag(&mut self) -> Result<u8> {
        loop {
            let tag = self.next_byte()?;
            if tag & TYPE_MASK != PAD {
                return Ok(tag);
            }
        }
    }

    fn read_f32(&mut self) -> Result<f32> {
        let buf = &self.input[self.pos..];
        if buf.len() < 4 {
            return Err(Error::UnexpectedEof);
        }
        self.pos += 4;
        Ok(LittleEndian::read_f32(buf))
    }

    fn read_f64(&mut self) -> Result<f64> {
        let buf = &self.input[self.pos..];
        if buf.len() < 8 {
            return Err(Error::UnexpectedEof);
        }
        self.pos += 8;
        Ok(LittleEndian::read_f64(buf))
    }

    fn read_varint(&mut self) -> Result<u64> {
        let mut cursor = io::Cursor::new(&self.input[self.pos..]);
        let val = cursor.read_varint()?;
        self.pos += cursor.position() as usize;
        Ok(val)
    }

    fn read_zigzag(&mut self) -> Result<i64> {
        let mut cursor = io::Cursor::new(&self.input[self.pos..]);
        let val = cursor.read_zigzag()?;
        self.pos += cursor.position() as usize;
        Ok(val)
    }

    fn read_bytes(&mut self, len: u64) -> Result<&'b [u8]> {
        if len > self.config.max_string_len() {
            return Err(Error::StringTooLarge(len));
        }

        if len > usize::max_value() as u64 {
            return Err(Error::StringTooLarge(len));
        }

        let a = self.pos;
        self.pos += len as usize;

        if self.pos > self.input.len() {
            return Err(Error::UnexpectedEof);
        }



        Ok(&self.input[a..self.pos])
    }

    pub fn tell(&mut self) -> Result<u64> {
        Ok((self.pos + 1) as u64)
    }

    pub fn seek(&mut self, pos: u64) -> Result<()> {
        self.pos = (pos - 1) as usize;
        Ok(())
    }
}
