use std::result;
use std::collections::HashMap;
use config::Config;
use reader::{self, Reader};

#[derive(Debug)]
pub enum Error {
    InvalidType,
    InvalidRef(usize),
    InvalidCopy,
    UnexpectedEof,
    OffsetOverflow,
    VarintOverflow,
    ArrayTooLarge { count: u64, limit: u64 },
    HashTooLarge { count: u64, limit: u64 },
}

impl Error {
    pub fn is_eof(&self) -> bool {
        match *self {
            Error::UnexpectedEof => true,
            _ => false,
        }
    }

    pub fn is_invalid_copy(&self) -> bool {
        match *self {
            Error::InvalidCopy => true,
            _ => false,
        }
    }
}

impl From<reader::Error> for Error {
    fn from(e: reader::Error) -> Error {
        match e {
            reader::Error::UnexpectedEof => Error::UnexpectedEof,
            reader::Error::OffsetOverflow => Error::OffsetOverflow,
            reader::Error::VarintOverflow => Error::VarintOverflow,
        }
    }
}

pub type Result<V> = result::Result<V, Error>;

pub trait Value<'buf>: Clone {
    type Array;
    type Hash;

    fn set_undef(&mut self);
    fn set_true(&mut self);
    fn set_false(&mut self);

    fn set_i64(&mut self, v: i64);
    fn set_u64(&mut self, v: u64);
    fn set_f32(&mut self, v: f32);
    fn set_f64(&mut self, v: f64);

    fn set_ref(&mut self, o: Self);
    fn set_weak_ref(&mut self, o: Self);
    fn set_alias(&mut self, o: Self);

    fn set_array(&mut self, a: Self::Array);
    fn set_hash(&mut self, h: Self::Hash);

    fn set_binary(&mut self, s: &'buf [u8]);
    fn set_string(&mut self, s: &'buf [u8]);

    fn set_object(&mut self, class: Self, value: Self) -> Result<()>;
    fn set_object_freeze(&mut self, class: Self, value: Self) -> Result<()>;
    fn set_regexp(&mut self, pattern: Self, flags: Self) -> Result<()>;
}

pub trait ArrayBuilder<'buf, V: Value<'buf>> {
    fn insert(&mut self, value: V) -> Result<()>;
    fn finalize(self) -> V::Array;
}

pub trait HashBuilder<'buf, V: Value<'buf>> {
    fn insert(&mut self, key: &'buf [u8], value: V) -> Result<()>;
    fn finalize(self) -> V::Hash;
}

pub trait Builder<'buf> {
    type Value: Value<'buf>;
    type ArrayBuilder: ArrayBuilder<'buf, Self::Value>;
    type HashBuilder: HashBuilder<'buf, Self::Value>;

    fn new(&mut self) -> Self::Value;

    fn build_array(&mut self, size: u64) -> Self::ArrayBuilder;
    fn build_hash(&mut self, size: u64) -> Self::HashBuilder;
}

pub struct Parser<'a, 'buf, B: Builder<'buf>> {
    config: &'a Config,
    reader: Reader<'buf>,
    track: HashMap<usize, B::Value>,
    builder: B,
    copy_pos: usize,
}

impl<'a, 'buf, B: Builder<'buf>> Parser<'a, 'buf, B> {
    pub fn new(builder: B, config: &'a Config, input: &'buf [u8]) -> Parser<'a, 'buf, B> {
        Parser {
            config: config,
            reader: Reader::new(input),
            track: HashMap::new(),
            builder: builder,
            copy_pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<B::Value> {
        self.parse_inner(false)
    }

    fn parse_str(&mut self) -> Result<&'buf [u8]> {
        use sereal_common::constants::*;

        let tag = self.reader.read_tag()?;
        let tag = tag & TYPE_MASK;

        match tag {
            SHORT_BINARY_0...SHORT_BINARY_31 => {
                let len = tag - SHORT_BINARY_0;
                Ok(self.reader.read_bytes(len.into())?)
            }

            STR_UTF8 | BINARY => {
                let len = self.reader.read_varlen()?;
                Ok(self.reader.read_bytes(len)?)
            }

            COPY => Ok(self.do_copy(|p| p.parse_str())?),

            _ => Err(Error::InvalidType),
        }
    }

    fn parse_inner(&mut self, force_track: bool) -> Result<B::Value> {
        use sereal_common::constants::*;

        let tag = self.reader.read_tag()?;

        let track = tag & TRACK_BIT != 0;
        let tag = tag & TYPE_MASK;

        let mut value = self.builder.new();

        if track || force_track {
            self.track.insert(self.reader.pos(), value.clone());
        }

        match tag {
            UNDEF | CANONICAL_UNDEF => value.set_undef(),

            POS_0...POS_15 => value.set_u64(tag as u64),
            NEG_16...NEG_1 => value.set_i64((tag | 0xf0) as i64),

            VARINT => value.set_u64(self.reader.read_varint()?),
            ZIGZAG => value.set_i64(self.reader.read_zigzag()?),
            FLOAT => value.set_f32(self.reader.read_f32()?),
            DOUBLE => value.set_f64(self.reader.read_f64()?),

            TRUE => value.set_true(),
            FALSE => value.set_false(),

            REFN => value.set_ref(self.parse()?),

            REFP => {
                let p = self.reader.read_varlen()?;
                value.set_ref(self.get(p)?);
            }

            ALIAS => {
                let p = self.reader.read_varlen()?;
                value.set_alias(self.get(p)?)
            }

            COPY => value.set_alias(self.do_copy(|p| p.parse())?),

            WEAKEN => value.set_weak_ref(self.parse()?),

            ARRAY => {
                let len = self.reader.read_varint()?;
                value.set_array(self.parse_array(len)?);
            }

            ARRAYREF_0...ARRAYREF_15 => {
                let len = tag - ARRAYREF_0;
                let array = self.parse_array(len as u64)?;
                let mut inner = self.builder.new();
                inner.set_array(array);
                value.set_ref(inner);
            }

            HASH => {
                let len = self.reader.read_varint()?;
                value.set_hash(self.parse_hash(len)?);
            }

            HASHREF_0...HASHREF_15 => {
                let len = tag - HASHREF_0;
                let hash = self.parse_hash(len as u64)?;
                let mut inner = self.builder.new();
                inner.set_hash(hash);
                value.set_ref(inner);
            }

            BINARY => {
                let len = self.reader.read_varlen()?;
                value.set_binary(self.reader.read_bytes(len)?);
            }

            STR_UTF8 => {
                let len = self.reader.read_varlen()?;
                value.set_string(self.reader.read_bytes(len)?);
            }

            SHORT_BINARY_0...SHORT_BINARY_31 => {
                let len = tag - SHORT_BINARY_0;
                value.set_binary(self.reader.read_bytes(len.into())?);
            }

            OBJECT => value.set_object(self.parse_inner(true)?, self.parse()?)?,

            OBJECTV => {
                let pos = self.reader.read_varlen()?;
                value.set_object(self.get(pos)?, self.parse()?)?;
            }

            OBJECT_FREEZE => {
                value.set_object_freeze(
                    self.parse_inner(true)?,
                    self.parse()?,
                )?
            }

            OBJECTV_FREEZE => {
                let pos = self.reader.read_varlen()?;
                value.set_object_freeze(self.get(pos)?, self.parse()?)?;
            }

            REGEXP => value.set_regexp(self.parse()?, self.parse()?)?,

            _ => unimplemented!(),
        };

        Ok(value)
    }

    fn get(&self, p: usize) -> Result<B::Value> {
        self.track.get(&p).cloned().ok_or(Error::InvalidRef(p))
    }

    fn do_copy<T, F: FnOnce(&mut Self) -> Result<T>>(&mut self, f: F) -> Result<T> {
        if self.copy_pos != 0 {
            return Err(Error::InvalidCopy);
        }

        let pos = self.reader.read_varlen()?;
        self.copy_pos = self.reader.set_pos(pos - 1);

        let val = f(self);

        self.reader.set_pos(self.copy_pos);
        self.copy_pos = 0;

        val
    }

    fn parse_array(&mut self, count: u64) -> Result<<B::Value as Value<'buf>>::Array> {
        if count > self.config.max_array_size() {
            return Err(Error::ArrayTooLarge {
                count: count,
                limit: self.config.max_array_size(),
            });
        }

        let mut v = self.builder.build_array(count);
        for _ in 0..count {
            let value = self.parse()?;
            v.insert(value)?;
        }
        Ok(v.finalize())
    }

    fn parse_hash(&mut self, count: u64) -> Result<<B::Value as Value<'buf>>::Hash> {
        if count > self.config.max_hash_size() {
            return Err(Error::HashTooLarge {
                count: count,
                limit: self.config.max_hash_size(),
            });
        }

        let old_copy_pos = self.copy_pos;
        self.copy_pos = 0;

        let mut m = self.builder.build_hash(count);
        for _ in 0..count {
            let k = self.parse_str()?;
            let v = self.parse()?;
            m.insert(k, v)?;
        }

        self.copy_pos = old_copy_pos;

        Ok(m.finalize())
    }
}

pub fn parse<'buf, B: Builder<'buf>>(s: &'buf [u8], builder: B) -> Result<B::Value> {
    let config = Config::default();
    let mut p = Parser::new(builder, &config, s);
    p.parse()
}
