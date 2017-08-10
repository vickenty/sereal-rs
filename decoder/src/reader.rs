use std::result;
use std::mem;
use byteorder::LittleEndian;
use byteorder::ByteOrder;
use sereal_common::constants::{ TYPE_MASK, PAD };
use varint;

pub enum Error {
    UnexpectedEof,
    OffsetOverflow,
    VarintOverflow,
}

pub type Result<T> = result::Result<T, Error>;

impl From<varint::Error> for Error {
    fn from(e: varint::Error) -> Error {
        match e {
            varint::Error::UnexpectedEof => Error::UnexpectedEof,
            varint::Error::Overflow => Error::VarintOverflow,
        }
    }
}

pub struct Reader<'buf> {
    input: &'buf [u8],
    pos: usize,
}

impl<'buf> Reader<'buf> {
    pub fn new(input: &'buf [u8]) -> Self {
        Reader {
            input: input,
            pos: 0,
        }
    }

    pub fn read_tag(&mut self) -> Result<u8> {
        loop {
            if self.pos >= self.input.len() {
                return Err(Error::UnexpectedEof);
            }

            let tag = self.input[self.pos];
            self.pos += 1;
            if tag & TYPE_MASK != PAD {
                return Ok(tag);
            }
        }
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        let buf = &self.input[self.pos..];
        if buf.len() < 4 {
            return Err(Error::UnexpectedEof);
        }
        self.pos += 4;
        Ok(LittleEndian::read_f32(buf))
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        let buf = &self.input[self.pos..];
        if buf.len() < 8 {
            return Err(Error::UnexpectedEof);
        }
        self.pos += 8;
        Ok(LittleEndian::read_f64(buf))
    }

    pub fn read_varint(&mut self) -> Result<u64> {
        let (val, len) = varint::parse_varint(&self.input[self.pos..])?;
        self.pos += len;
        Ok(val)
    }

    pub fn read_zigzag(&mut self) -> Result<i64> {
        let (val, len) = varint::parse_zigzag(&self.input[self.pos..])?;
        self.pos += len;
        Ok(val)
    }

    pub fn read_varlen(&mut self) -> Result<usize> {
        let len = self.read_varint()?;
        if len < usize::max_value() as u64 {
            Ok(len as usize)
        } else {
            Err(Error::OffsetOverflow)
        }
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'buf [u8]> {
        let beg = self.pos;
        self.pos = self.pos.checked_add(len).ok_or(Error::OffsetOverflow)?;
        if self.pos <= self.input.len() {
            Ok(&self.input[beg..self.pos])
        } else {
            Err(Error::UnexpectedEof)
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn set_pos(&mut self, new: usize) -> usize {
        mem::replace(&mut self.pos, new)
    }
}
