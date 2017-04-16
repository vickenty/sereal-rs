use std::io;
use std::result;

use byteorder::{ LittleEndian, ReadBytesExt };
use sereal_common::constants::*;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    VarintOverflow,
    UnknownTag(u8),
}

impl Error {
    pub fn is_eof(&self) -> bool {
        match *self {
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

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Tag {
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
    Array(u64),
    ArrayRef(u8),
    Hash(u64),
    HashRef(u8),
    Bin(Vec<u8>),
    Str(Vec<u8>),
    Object,
    ObjectV(u64),
    ObjectFreeze,
    ObjectVFreeze(u64),
    Weaken,
    Regexp,
}

pub struct Token {
    pub pos: u64,
    pub track: bool,
    pub tag: Tag,
}

pub struct Lexer<R> {
    input: R,
}

impl<R: io::Read + io::Seek> Lexer<R> {
    pub fn next(&mut self) -> Result<Token> {
        let tag = self.read_tag()?;
        let pos = self.input.seek(io::SeekFrom::Current(0))?;
        let trk = tag & TRACK_BIT == TRACK_BIT;
        let tag = tag & TYPE_MASK;

        let value = match tag & TYPE_MASK {
            POS_0...
            POS_15 => Tag::Pos(tag),

            NEG_16...
            NEG_1 => Tag::Neg((tag | 0xf0) as i8),

            VARINT => Tag::Varint(self.read_varint()?),
            ZIGZAG => Tag::Zigzag(self.read_zigzag()?),

            FLOAT => Tag::Float(self.input.read_f32::<LittleEndian>()?),
            DOUBLE => Tag::Double(self.input.read_f64::<LittleEndian>()?),
            LONG_DOUBLE => unimplemented!(),

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
            COPY => self.read_copy()?,
            WEAKEN => Tag::Weaken,
            REGEXP => Tag::Regexp,

            OBJECT_FREEZE => Tag::ObjectFreeze,
            OBJECTV_FREEZE => Tag::ObjectVFreeze(self.read_varint()?),

            RESERVED_0...
            RESERVED_4 => return Err(Error::UnknownTag(tag)),

            CANONICAL_UNDEF => Tag::CanonicalUndef,
            FALSE => Tag::False,
            TRUE => Tag::True,
            MANY => unimplemented!(),
            PACKET_START => return Err(Error::UnknownTag(tag)),
            EXTEND => return Err(Error::UnknownTag(tag)),

            PAD => panic!("PAD should be handled in read_tag()"),

            ARRAYREF_0...
            ARRAYREF_15 => Tag::ArrayRef(tag - ARRAYREF_0),

            HASHREF_0...
            HASHREF_15 => Tag::HashRef(tag - HASHREF_0),

            SHORT_BINARY_0 => Tag::Bin(Vec::new()),
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
}

impl<R> Lexer<R> where R: io::Read + io::Seek {
    pub fn new(r: R) -> Lexer<R> {
        Lexer {
            input: r
        }
    }

    fn read_tag(&mut self) -> Result<u8> {
        loop {
            let tag = self.input.read_u8()?;
            if tag & TYPE_MASK != PAD {
                return Ok(tag);
            }
        }
    }

    fn read_varint(&mut self) -> Result<u64> {
        let mut a = 0;
        let mut o = 0;

        loop {
            let i = self.input.read_u8()?;
            a |= ((i & 0x7f) as u64) << o;

            if i & 0x80 == 0 {
                return Ok(a);
            }

            o += 7;
            if o >= 64 {
                return Err(Error::VarintOverflow);
            }
        }
    }

    fn read_zigzag(&mut self) -> Result<i64> {
        let v = self.read_varint()?;
        Ok((v >> 1) as i64 ^ -((v & 1) as i64))
    }

    fn read_bytes(&mut self, len: u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; len as usize];
        self.input.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_copy(&mut self) -> Result<Tag> {
        let offset = self.read_varint()?;
        let current = self.input.seek(io::SeekFrom::Current(0))?;
        self.input.seek(io::SeekFrom::Start(offset))?;
        let token = self.next()?;
        self.input.seek(io::SeekFrom::Start(current))?;
        Ok(token.tag)
    }
}

pub fn from_slice(s: &[u8]) -> Lexer<io::Cursor<&[u8]>> {
    Lexer::new(io::Cursor::new(s))
}

#[test]
fn test_varint() {
    use std::u64::MAX;

    fn r(s: &[u8]) -> Result<u64> {
        from_slice(s).read_varint()
    }

    fn t(s: &[u8]) -> u64 {
        r(s).unwrap()
    }

    fn e(s: &[u8]) -> Error {
        r(s).unwrap_err()
    }

    assert_eq!(t(b"\x00"), 0);
    assert_eq!(t(b"\x01"), 1);
    assert_eq!(t(b"\x80\x01"), 128);
    assert_eq!(t(b"\x80\x80\x01"), 16384);
    assert_eq!(t(b"\x81\x01"), 129);
    assert_eq!(t(b"\x81\x81\x00"), 129);
    assert_eq!(t(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00"), 0);
    assert_eq!(t(b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\x7f"), MAX);

    assert!(e(b"\x80").is_eof());
    assert!(e(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00").is_varint_overflow());
}

#[test]
fn test_zigzag() {
    use std::i64::{ MIN, MAX };

    fn r(s: &[u8]) -> Result<i64> {
        from_slice(s).read_zigzag()
    }

    fn t(s: &[u8]) -> i64 {
        r(s).unwrap()
    }

    fn e(s: &[u8]) -> Error {
        r(s).unwrap_err()
    }

    assert_eq!(t(b"\x00"), 0);
    assert_eq!(t(b"\x01"), -1);
    assert_eq!(t(b"\x02"), 1);
    assert_eq!(t(b"\x03"), -2);
    assert_eq!(t(b"\x04"), 2);
    assert_eq!(t(b"\x80\x01"), 64);
    assert_eq!(t(b"\x80\x80\x01"), 8192);
    assert_eq!(t(b"\x81\x01"), -65);
    assert_eq!(t(b"\x81\x81\x00"), -65);
    assert_eq!(t(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00"), 0);
    assert_eq!(t(b"\xfe\xff\xff\xff\xff\xff\xff\xff\xff\x7f"), MAX);
    assert_eq!(t(b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\x7f"), MIN);

    assert!(e(b"\x80").is_eof());
    assert!(e(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00").is_varint_overflow());
}