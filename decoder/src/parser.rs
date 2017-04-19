use std::io;
use std::result;
use std::collections::HashMap;

use config::Config;
use lexer;
use lexer::Lexer;
use lexer::Tag;

#[derive(Debug)]
pub enum Error {
    InvalidType,
    InvalidRef(u64),
    InvalidCopy,
    ArrayTooLarge { count: u64, limit: u64 },
    HashTooLarge { count: u64, limit: u64 },
    LexerError(lexer::Error),
}

impl Error {
    pub fn is_eof(&self) -> bool {
        match *self {
            Error::LexerError(ref e) => e.is_eof(),
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

impl From<lexer::Error> for Error {
    fn from(e: lexer::Error) -> Self {
        Error::LexerError(e)
    }
}

pub type Result<V> = result::Result<V, Error>;

pub trait Value {
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

    fn set_binary(&mut self, s: &[u8]);
    fn set_string(&mut self, s: &[u8]);

    fn set_object(&mut self, class: Self, value: Self) -> Result<()>;
    fn set_object_freeze(&mut self, class: Self, value: Self) -> Result<()>;
    fn set_regexp(&mut self, pattern: Self, flags: Self) -> Result<()>;
}

pub trait ArrayBuilder<V: Value> {
    fn insert(&mut self, value: V) -> Result<()>;
    fn finalize(self) -> V::Array;
}

pub trait HashBuilder<V: Value> {
    fn insert(&mut self, key: V, value: V) -> Result<()>;
    fn finalize(self) -> V::Hash;
}

pub trait Builder {
    type Value: Value + Clone;
    type ArrayBuilder: ArrayBuilder<Self::Value>;
    type HashBuilder: HashBuilder<Self::Value>;

    fn new(&mut self) -> Self::Value;

    fn build_array(&mut self, size: u64) -> Self::ArrayBuilder;
    fn build_hash(&mut self, size: u64) -> Self::HashBuilder;
}

pub struct Parser<'a, R, B: Builder> {
    config: &'a Config,
    lexer: Lexer<'a, R>,
    track: HashMap<u64, B::Value>,
    builder: B,
    copy_pos: u64,
}

impl<'a, R: io::Read + io::Seek, B: Builder> Parser<'a, R, B> {
    pub fn new(reader: R, builder: B, config: &'a Config) -> Parser<'a, R, B> {
        Parser {
            lexer: Lexer::new(reader, config),
            config: config,
            track: HashMap::new(),
            builder: builder,
            copy_pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<B::Value> {
        self.parse_inner(false)
    }

    fn parse_str(&mut self, force_track: bool) -> Result<B::Value> {
        let saved = self.copy_pos;
        self.copy_pos = 0;
        let res = self.parse_inner(force_track);
        self.copy_pos = saved;
        res
    }

    fn parse_inner(&mut self, force_track: bool) -> Result<B::Value> {
        let token = self.lexer.next()?;

        let mut value = self.builder.new();

        if token.track || force_track {
            self.track.insert(token.pos, value.clone());
        }

        match token.tag {
            Tag::Undef => value.set_undef(),
            Tag::CanonicalUndef => value.set_undef(),

            Tag::Pos(v) => value.set_u64(v as u64),
            Tag::Neg(v) => value.set_i64(v as i64),
            Tag::Varint(v) => value.set_u64(v),
            Tag::Zigzag(v) => value.set_i64(v),
            Tag::Float(v) => value.set_f32(v),
            Tag::Double(v) => value.set_f64(v),

            Tag::True => value.set_true(),
            Tag::False => value.set_false(),

            Tag::Refn => value.set_ref(self.parse()?),
            Tag::Refp(p) => value.set_ref(self.get(p)?),
            Tag::Alias(p) => value.set_alias(self.get(p)?),
            Tag::Copy(pos) => {
                self.enter_copy(pos)?;
                let val = self.parse()?;
                self.leave_copy()?;

                value.set_alias(val);
            },
            Tag::Weaken => value.set_weak_ref(self.parse()?),

            Tag::Array(c) => value.set_array(self.parse_array(c)?),
            Tag::ArrayRef(c) => {
                let array = self.parse_array(c as u64)?;
                let mut inner = self.builder.new();
                inner.set_array(array);
                value.set_ref(inner);
            },
            Tag::Hash(c) => value.set_hash(self.parse_hash(c)?),
            Tag::HashRef(c) => {
                let hash = self.parse_hash(c as u64)?;
                let mut inner = self.builder.new();
                inner.set_hash(hash);
                value.set_ref(inner);
            },

            Tag::Bin(v) => value.set_binary(&v),
            Tag::Str(v) => value.set_string(&v),

            Tag::Object => value.set_object(self.parse_str(true)?, self.parse()?)?,
            Tag::ObjectV(o) => value.set_object(self.get(o)?, self.parse()?)?,
            Tag::ObjectFreeze => value.set_object_freeze(self.parse_str(true)?, self.parse()?)?,
            Tag::ObjectVFreeze(o) => value.set_object_freeze(self.get(o)?, self.parse()?)?,

            Tag::Regexp => value.set_regexp(self.parse()?, self.parse()?)?,
        };

        Ok(value)
    }

    fn get(&self, p: u64) -> Result<B::Value> {
        self.track.get(&p).cloned().ok_or(Error::InvalidRef(p))
    }

    fn parse_array(&mut self, count: u64) -> Result<<B::Value as Value>::Array> {
        if count > self.config.max_array_size() {
            return Err(Error::ArrayTooLarge { count: count, limit: self.config.max_array_size() });
        }

        let mut v = self.builder.build_array(count);
        for _ in 0..count {
            let value = self.parse()?;
            v.insert(value)?;
        }
        Ok(v.finalize())
    }

    fn parse_hash(&mut self, count: u64) -> Result<<B::Value as Value>::Hash> {
        if count > self.config.max_hash_size() {
            return Err(Error::HashTooLarge { count: count, limit: self.config.max_hash_size() });
        }

        let mut m = self.builder.build_hash(count);
        for _ in 0..count {
            let k = self.parse_str(false)?;
            let v = self.parse()?;
            m.insert(k, v)?;
        }
        Ok(m.finalize())
    }

    fn enter_copy(&mut self, pos: u64) -> Result<()> {
        if self.copy_pos != 0 {
            return Err(Error::InvalidCopy);
        }

        self.copy_pos = self.lexer.tell()?;
        self.lexer.seek(pos)?;
        Ok(())
    }

    fn leave_copy(&mut self) -> Result<()> {
        self.lexer.seek(self.copy_pos)?;
        self.copy_pos = 0;
        Ok(())
    }
}

pub fn parse<B: Builder>(s: &[u8], builder: B) -> Result<B::Value> {
    let config = Config::default();
    let mut p = Parser::new(io::Cursor::new(s), builder, &config);
    p.parse()
}
