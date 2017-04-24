use std::error;
use std::fmt;
use std::collections::BTreeSet;

use serde::de;

use lexer;
use lexer::Lexer;
use lexer::Tag;
use config::Config;

pub enum Error {
    InvalidRef(u64),
    Custom(String),
    Lexer(lexer::Error),
}

impl Error {
    pub fn as_invalid_ref(&self) -> Option<u64> {
        match self {
            &Error::InvalidRef(p) => Some(p),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::InvalidRef(p) => write!(f, "invalid reference {}", p),
            &Error::Custom(ref b) => write!(f, "{}", b),
            &Error::Lexer(ref l) => write!(f, "lexer error: {:?}", l),
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
        match self {
            &Error::InvalidRef(_) => "invalid reference",
            &Error::Custom(_) => "custom error",
            &Error::Lexer(_) => "lexing error",
        }
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::Custom(format!("{}", msg))
    }
}

impl From<lexer::Error> for Error {
    fn from(e: lexer::Error) -> Error {
        Error::Lexer(e)
    }
}

pub struct Deserializer<'cfg, 'b> {
    lexer: Lexer<'cfg, 'b>,
    seen: BTreeSet<u64>,
}

impl<'cfg, 'b> Deserializer<'cfg, 'b> {
    pub fn new(config: &'cfg Config, input: &'b [u8]) -> Self {
        Deserializer {
            lexer: Lexer::new(config, input),
            seen: BTreeSet::new(),
        }
    }
}

impl<'cfg, 'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'cfg, 'de> {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value, Error> {
        let token = self.lexer.next()?;

        match token.tag {
            Tag::Pos(v) => visitor.visit_u8(v),
            Tag::Neg(v) => visitor.visit_i8(v),
            Tag::Float(v) => visitor.visit_f32(v),
            Tag::Double(v) => visitor.visit_f64(v),
            Tag::Bin(v) => visitor.visit_borrowed_bytes(v),
            Tag::Str(v) => visitor.visit_borrowed_bytes(v),
            Tag::Array(v) => visitor.visit_seq(Seq::new(self, v)),
            Tag::ArrayRef(v) => visitor.visit_seq(Seq::new(self, v as u64)),
            Tag::Undef => visitor.visit_none(),
            Tag::Refn => visitor.visit_some(self),
            Tag::Refp(p) => {
                let cur = self.lexer.tell()?;

                if self.seen.contains(&p) || p >= cur {
                    return Err(Error::InvalidRef(p));
                }

                self.seen.insert(p);
                self.lexer.seek(p)?;

                let res = visitor.visit_some(&mut *self);

                self.lexer.seek(cur)?;
                self.seen.remove(&p);

                res
            }
            Tag::Hash(v) => visitor.visit_map(Map::new(self, v)),
            Tag::HashRef(v) => visitor.visit_map(Map::new(self, v as u64)),
            _ => unimplemented!(),
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
    fn deserialize_unit_struct<V: de::Visitor<'de>>(self, _: &'static str, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(self, _: &'static str, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_seq<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_tuple<V: de::Visitor<'de>>(self, _: usize, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(self, _: &'static str, _: usize, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_map<V: de::Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_struct<V: de::Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], v: V) -> Result<V::Value, Error> {
        self.deserialize_any(v)
    }
    fn deserialize_enum<V: de::Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], v: V) -> Result<V::Value, Error> {
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

    fn next_element_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Error> {
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

    fn next_key_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Error> {
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
        assert_eq!(Option::<Vec<u64>>::de(b"\x28\x2b\x03\x01\x02\x03"), Some(vec![1, 2, 3]));
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

        assert_eq!(S::de(b"\x43\x01\x02\x43\x03\x04\x05"), S(1, 2, vec![3, 4, 5]));
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
        assert_eq!(S::de(b"\x42\x28\x50\x25"), S { f: s.clone(), g: None });
        assert_eq!(S::de(b"\x42\x28\xd0\x29\x03"), S { f: s.clone(), g: s.clone() });
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

        assert_eq!(s, S { a: "foo", b: "bar", c: "baz" });
    }
}
