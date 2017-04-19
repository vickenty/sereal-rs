use std::io;
use std::error;
use std::fmt;

use serde::de;

use lexer;
use lexer::{ Lexer, Tag };
use config::Config;

pub enum Error {
    InvalidRef(u64),
    Custom(String),
    Lexer(lexer::Error),
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

//#[derive(Clone, Debug, PartialEq)]
//enum Inner<'a> {
//    Undef,
//    I64(i64),
//    U64(u64),
//    F32(f32),
//    F64(f64),
//    String(Vec<u8>),
//    Ref(Value<'a>),
//    WeakRef(Value<'a>),
//    Array(Vec<Value<'a>>),
//    Hash(Vec<(Value<'a>, Value<'a>)>),
//    Object(Value<'a>, Value<'a>),
//    Bool(bool),
//    Regexp(Value<'a>, Value<'a>),
//}
//
//#[derive(Copy, Clone, Debug, PartialEq)]
//struct Value<'a>(&'a RefCell<Inner<'a>>);

pub struct Deserializer<'a, R> {
    lexer: Lexer<'a, R>,
}

impl<'a, R: io::Read + io::Seek> Deserializer<'a, R> {
    pub fn new(reader: R, config: &'a Config) -> Self {
        Deserializer {
            lexer: Lexer::new(reader, config),
        }
    }
}

impl<'a, 'b, R: io::Read + io::Seek> de::Deserializer for &'b mut Deserializer<'a, R> {
    type Error = Error;

    fn deserialize<V: de::Visitor>(mut self, visitor: V) -> Result<V::Value, Error> {
        let token = self.lexer.next()?;

        match token.tag {
            Tag::Pos(v) => visitor.visit_u8(v),
            Tag::Neg(v) => visitor.visit_i8(v),
            Tag::Float(v) => visitor.visit_f32(v),
            Tag::Double(v) => visitor.visit_f64(v),
            Tag::Bin(v) => visitor.visit_byte_buf(v),
            Tag::Str(v) => visitor.visit_byte_buf(v),
            Tag::Array(v) => visitor.visit_seq(Seq::new(self, v)),
            Tag::ArrayRef(v) => visitor.visit_seq(Seq::new(self, v as u64)),
            Tag::Refn => self.deserialize(visitor),
            Tag::Hash(v) => visitor.visit_map(Map::new(self, v)),
            Tag::HashRef(v) => visitor.visit_map(Map::new(self, v as u64)),
            _ => unimplemented!(),
        }
    }

    fn deserialize_bool<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_u8<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_u16<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_u32<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_u64<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_i8<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_i16<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_i32<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_i64<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_f32<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_f64<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_char<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_str<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_string<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_bytes<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_byte_buf<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_option<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_unit<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_unit_struct<V: de::Visitor>(self, _: &'static str, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_newtype_struct<V: de::Visitor>(self, _: &'static str, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_seq<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_seq_fixed_size<V: de::Visitor>(self, _: usize, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_tuple<V: de::Visitor>(self, _: usize, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_tuple_struct<V: de::Visitor>(self, _: &'static str, _: usize, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_map<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_struct<V: de::Visitor>(self, _: &'static str, _: &'static [&'static str], v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_struct_field<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_enum<V: de::Visitor>(self, _: &'static str, _: &'static [&'static str], v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
    fn deserialize_ignored_any<V: de::Visitor>(self, v: V) -> Result<V::Value, Error> {
        self.deserialize(v)
    }
}

struct Seq<'b, 'a: 'b, R: 'b> {
    de: &'b mut Deserializer<'a, R>,
    count: u64,
}

impl<'de, 'a, R> Seq<'de, 'a, R> {
    fn new(de: &'de mut Deserializer<'a, R>, count: u64) -> Seq<'de, 'a, R> {
        Seq {
            de: de,
            count: count,
        }
    }
}

impl<'de, 'a, R: io::Read + io::Seek> de::SeqVisitor for Seq<'de, 'a, R> {
    type Error = Error;

    fn visit_seed<T: de::DeserializeSeed>(&mut self, seed: T) -> Result<Option<T::Value>, Error> {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        Ok(Some(seed.deserialize(&mut *self.de)?))
    }
}

struct Map<'b, 'a: 'b, R: 'b> {
    de: &'b mut Deserializer<'a, R>,
    count: u64,
}

impl<'b, 'a, R> Map<'b, 'a, R> {
    fn new(de: &'b mut Deserializer<'a, R>, count: u64) -> Map<'b, 'a, R> {
        Map {
            de: de,
            count: count,
        }
    }
}

impl<'b, 'a, R: io::Read + io::Seek> de::MapVisitor for Map<'b, 'a, R> {
    type Error = Error;

    fn visit_key_seed<T: de::DeserializeSeed>(&mut self, seed: T) -> Result<Option<T::Value>, Error> {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        Ok(Some(seed.deserialize(&mut *self.de)?))
    }

    fn visit_value_seed<T: de::DeserializeSeed>(&mut self, seed: T) -> Result<T::Value, Error> {
        Ok(seed.deserialize(&mut *self.de)?)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use std::collections::HashMap;

    use serde::de::Deserialize;

    use config::Config;
    use super::Deserializer;

    trait De {
        fn de(s: &[u8]) -> Self;
    }

    impl<T: Deserialize> De for T {
        fn de(s: &[u8]) -> T {
            let config = Config::default();
            let mut de = Deserializer::new(Cursor::new(s), &config);
            T::deserialize(&mut de).unwrap()
        }
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Tuple(u32, u64, Vec<u8>);

    #[derive(Deserialize, PartialEq, Debug)]
    struct Named {
        foo: u32,
        bar: Vec<u8>,
        baz: String,
    }

    #[test]
    fn deserialize_int() {
        assert_eq!(u64::de(b"\x01"), 1);
    }

    #[test]
    fn deserialize_vec() {
        assert_eq!(Vec::<i32>::de(b"\x43\x01\x02\x03"), vec![1, 2, 3]);
        assert_eq!(Vec::<u64>::de(b"\x28\x2b\x03\x01\x02\x03"), vec![1, 2, 3]);
    }

    #[test]
    fn deserialize_map() {
        let mut map = HashMap::new();
        map.insert(1, 2);
        map.insert(3, 4);

        assert_eq!(HashMap::<u32, u32>::de(b"\x52\x01\x02\x03\x04\x05"), map);
    }

    #[test]
    fn deserialize_tuple() {
        assert_eq!(Tuple::de(b"\x43\x01\x02\x43\x03\x04\x05"), Tuple(1, 2, vec![3, 4, 5]));
    }

    #[test]
    fn deserialize_named() {
        assert_eq!(
            Named::de(b"\x53\x63foo\x01\x63bar\x42\x02\x03\x63baz\x62ok"),
            Named {
                foo: 1,
                bar: vec![2, 3],
                baz: "ok".to_owned(),
            }
        );
    }
}
