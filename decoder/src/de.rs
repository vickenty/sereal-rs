use std::error;
use std::fmt;
use std::collections::BTreeSet;

use serde::de;
use sereal_common::constants::*;

use config::Config;
use reader::{self, Reader};

pub enum Error {
    UnexpectedEof,
    OffsetOverflow,
    VarintOverflow,
    InvalidRef(usize),
    Custom(String),
}

impl Error {
    pub fn as_invalid_ref(&self) -> Option<usize> {
        match self {
            &Error::InvalidRef(p) => Some(p),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            UnexpectedEof | OffsetOverflow | VarintOverflow => {
                write!(f, "{}", error::Error::description(self))
            }
            InvalidRef(p) => write!(f, "invalid reference {}", p),
            Custom(ref b) => write!(f, "{}", b),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            UnexpectedEof => "unexpected eof",
            OffsetOverflow => "offset overflow",
            VarintOverflow => "varint overflow",
            InvalidRef(_) => "invalid reference",
            Custom(_) => "custom error",
        }
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::Custom(format!("{}", msg))
    }
}

impl From<reader::Error> for Error {
    fn from(e: reader::Error) -> Error {
        match e {
            reader::Error::UnexpectedEof => Error::UnexpectedEof,
            reader::Error::OffsetOverflow => Error::OffsetOverflow,
            reader::Error::VarintOverflow => Error::OffsetOverflow,
        }
    }
}

pub struct Deserializer<'cfg, 'b> {
    config: &'cfg Config,
    reader: Reader<'b>,
    seen: BTreeSet<usize>,
}

impl<'cfg, 'b> Deserializer<'cfg, 'b> {
    pub fn new(config: &'cfg Config, input: &'b [u8]) -> Self {
        Deserializer {
            config: config,
            reader: Reader::new(input),
            seen: BTreeSet::new(),
        }
    }
}

impl<'cfg, 'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'cfg, 'de> {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Error> {
        let tag = self.reader.read_tag()? & TYPE_MASK;

        match tag {
            POS_0...POS_15 => visitor.visit_u8(tag),
            NEG_16...NEG_1 => visitor.visit_i8((tag | 0xf0) as i8),
            FLOAT => visitor.visit_f32(self.reader.read_f32()?),
            DOUBLE => visitor.visit_f64(self.reader.read_f64()?),

            BINARY | STR_UTF8 => {
                let len = self.reader.read_varlen()?;
                visitor.visit_borrowed_bytes(self.reader.read_bytes(len)?)
            }

            SHORT_BINARY_0...SHORT_BINARY_31 => {
                let len = tag - SHORT_BINARY_0;
                visitor.visit_borrowed_bytes(self.reader.read_bytes(len as usize)?)
            }

            ARRAY => {
                let len = self.reader.read_varint()?;
                visitor.visit_seq(Seq::new(self, len))
            }

            ARRAYREF_0...ARRAYREF_15 => {
                let len = tag - ARRAYREF_0;
                visitor.visit_seq(Seq::new(self, len as u64))
            }

            UNDEF | CANONICAL_UNDEF => visitor.visit_none(),

            REFN => visitor.visit_some(self),

            REFP => {
                let p = self.reader.read_varlen()?;

                if self.seen.contains(&p) || p >= self.reader.pos() {
                    return Err(Error::InvalidRef(p));
                }

                self.seen.insert(p);
                let prev = self.reader.set_pos(p - 1);

                let res = visitor.visit_some(&mut *self);

                self.reader.set_pos(prev);
                self.seen.remove(&p);

                res
            }

            HASHREF_0...HASHREF_15 => {
                let len = tag - HASHREF_0;
                visitor.visit_map(Map::new(self, len as u64))
            }

            _ => {
                panic!(
                    "tag type {tag} (0x{tag:02x}) not implemented yet",
                    tag = tag
                )
            }
        }
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_u8<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_u16<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_u32<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_u64<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_i8<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_i16<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_i32<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_i64<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_f32<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_f64<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_char<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_str<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_string<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_bytes<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_option<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_unit<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _: &'static str,
        v: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _: &'static str,
        v: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_seq<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_tuple<V: de::Visitor<'de>>(self, _: usize, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _: &'static str,
        _: usize,
        v: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_map<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        v: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        v: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_identifier<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
}

struct Seq<'a, 'cfg: 'a, 'de: 'a> {
    de: &'a mut Deserializer<'cfg, 'de>,
    count: u64,
}

impl<'a, 'cfg, 'de> Seq<'a, 'cfg, 'de> {
    fn new(de: &'a mut Deserializer<'cfg, 'de>, count: u64) -> Seq<'a, 'cfg, 'de> {
        Seq {
            de: de,
            count: count,
        }
    }
}

impl<'de, 'a, 'cfg> de::SeqAccess<'de> for Seq<'a, 'cfg, 'de> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        Ok(Some(seed.deserialize(&mut *self.de)?))
    }
}

struct Map<'a, 'cfg: 'a, 'de: 'a> {
    de: &'a mut Deserializer<'cfg, 'de>,
    count: u64,
}

impl<'a, 'cfg, 'de> Map<'a, 'cfg, 'de> {
    fn new(de: &'a mut Deserializer<'cfg, 'de>, count: u64) -> Map<'a, 'cfg, 'de> {
        Map {
            de: de,
            count: count,
        }
    }
}

impl<'de, 'a, 'cfg> de::MapAccess<'de> for Map<'a, 'cfg, 'de> {
    type Error = Error;

    fn next_key_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        Ok(Some(seed.deserialize(&mut *self.de)?))
    }

    fn next_value_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value, Error> {
        Ok(seed.deserialize(&mut *self.de)?)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::fmt::Debug;

    use serde::de::Deserialize;

    use config::Config;
    use super::Deserializer;
    use super::Error;

    trait De<'de>: Debug + Sized {
        fn de_res(s: &'de [u8]) -> Result<Self, Error>;

        fn de(s: &'de [u8]) -> Self {
            Self::de_res(s).unwrap()
        }

        fn err(s: &'de [u8]) -> Error {
            Self::de_res(s).unwrap_err()
        }
    }

    impl<'de, T: Debug + Deserialize<'de>> De<'de> for T {
        fn de_res(s: &'de [u8]) -> Result<T, Error> {
            let config = Config::default();
            let mut de = Deserializer::new(&config, s);
            T::deserialize(&mut de)
        }
    }

    #[test]
    fn ints() {
        assert_eq!(u64::de(b"\x01"), 1);
    }

    #[test]
    fn vecs() {
        assert_eq!(Vec::<i32>::de(b"\x43\x01\x02\x03"), vec![1, 2, 3]);
        assert_eq!(
            Option::<Vec<u64>>::de(b"\x28\x2b\x03\x01\x02\x03"),
            Some(vec![1, 2, 3])
        );
    }

    #[test]
    fn hashmaps() {
        let mut map = HashMap::new();
        map.insert(1, 2);
        map.insert(3, 4);

        assert_eq!(HashMap::<u32, u32>::de(b"\x52\x01\x02\x03\x04\x05"), map);
    }

    #[test]
    fn tuples() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct S(u32, u64, Vec<u8>);

        assert_eq!(
            S::de(b"\x43\x01\x02\x43\x03\x04\x05"),
            S(1, 2, vec![3, 4, 5])
        );
    }

    #[test]
    fn structs() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct S {
            foo: u32,
            bar: Vec<u8>,
            baz: String,
        }

        assert_eq!(
            S::de(b"\x53\x63foo\x01\x63bar\x42\x02\x03\x63baz\x62ok"),
            S {
                foo: 1,
                bar: vec![2, 3],
                baz: "ok".to_owned(),
            }
        );
    }

    #[test]
    fn refs() {
        #[derive(Deserialize, Debug, PartialEq, Clone)]
        struct S {
            f: Option<Box<S>>,
            g: Option<Box<S>>,
        };

        let s = Some(Box::new(S { f: None, g: None }));

        assert_eq!(S::de(b"\x50"), S { f: None, g: None });
        assert_eq!(
            S::de(b"\x42\x28\x50\x25"),
            S {
                f: s.clone(),
                g: None,
            }
        );
        assert_eq!(
            S::de(b"\x42\x28\xd0\x29\x03"),
            S {
                f: s.clone(),
                g: s.clone(),
            }
        );
        assert_eq!(S::err(b"\x42\x29\x01\x28\x50").as_invalid_ref(), Some(1));
        assert_eq!(S::err(b"\x42\x28\x50\x29\x01").as_invalid_ref(), Some(1));
    }

    #[test]
    fn borrow_str() {
        #[derive(Deserialize, Debug, PartialEq, Clone)]
        struct S<'a> {
            a: &'a str,
            b: &'a str,
            c: &'a str,
        }

        let d = b"\x43\x63foo\x63bar\x63baz";
        let s = S::de(&d[..]);

        assert_eq!(
            s,
            S {
                a: "foo",
                b: "bar",
                c: "baz",
            }
        );
    }
}
