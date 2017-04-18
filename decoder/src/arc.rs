use std::collections::{ HashSet, HashMap };
use std::fmt;
use std::sync::*;

use parser;
pub use parser::Error;
pub use parser::Result;

#[derive(Clone, Debug, PartialEq)]
pub enum Inner {
    Undef,
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(Vec<u8>),
    Ref(Value),
    WeakRef(Value),
    Array(Vec<Value>),
    Hash(HashMap<Vec<u8>, Value>),
    Object(Vec<u8>, Value),
    Bool(bool),
    Regexp(Vec<u8>, Vec<u8>),
}

#[derive(Clone)]
pub enum Value {
    Strong(Arc<RwLock<Inner>>),
    Weak(Weak<RwLock<Inner>>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f, &mut HashSet::new())
    }
}

impl Value {
    pub fn new(v: Inner) -> Value {
        Value::Strong(Arc::new(RwLock::new(v)))
    }

    fn read(&self) -> Inner {
        match self {
            &Value::Strong(ref a) => (&*a.read().unwrap()).clone(),
            &Value::Weak(ref w) => match w.upgrade() {
                Some(ref l) => (&*l.read().unwrap()).clone(),
                None => Inner::Undef,
            }
        }
    }

    /// Update the inner value.
    fn set(&self, v: Inner) {
        match self {
            &Value::Strong(ref a) => { *a.write().unwrap() = v },
            &Value::Weak(ref w) => {
                let a = w.upgrade().expect("writing expired weak-ref");
                let mut l = a.write().unwrap();
                *l = v;
            },
        }
    }

    /// Make this cell to alias another. Old value is discarded.
    fn alias(&mut self, v: Value) {
        *self = v;
    }

    fn to_string(&self) -> Result<Vec<u8>> {
        self.read().to_string()
    }

    fn downgrade(self) -> Value {
        match self {
            Value::Strong(a) => Value::Weak(Arc::downgrade(&a)),
            _ => self,
        }
    }

    fn upgrade(&self) -> Option<Arc<RwLock<Inner>>> {
        match self {
            &Value::Strong(ref a) => Some(a.clone()),
            &Value::Weak(ref w) => w.upgrade(),
        }
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter, seen: &mut HashSet<usize>) -> fmt::Result {
        match self.upgrade() {
            Some(ref a) => {
                let obj_id = a.as_ref() as *const _ as usize;
                if seen.contains(&obj_id) {
                    write!(f, "<loop>")
                } else {
                    seen.insert(obj_id);
                    (&*a.read().unwrap()).debug_fmt(f, seen)
                }
            },

            None => write!(f, "<dead weak ref>")
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        self.read() == other.read()
    }
}

impl From<i64> for Inner {
    fn from(v: i64) -> Inner {
        Inner::I64(v)
    }
}

impl From<u64> for Inner {
    fn from(v: u64) -> Inner {
        Inner::U64(v)
    }
}

impl<T> From<T> for Value where Inner: From<T> {
    fn from(v: T) -> Value {
        Value::new(Inner::from(v))
    }
}

impl<'a> From<&'a [u8]> for Inner {
    fn from(v: &'a [u8]) -> Inner {
        Inner::String(v.to_owned())
    }
}

impl Inner {
    fn to_string(&self) -> Result<Vec<u8>> {
        match self {
            &Inner::String(ref v) => Ok(v.clone()),
            _ => Err(Error::InvalidType),
        }
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter, seen: &mut HashSet<usize>) -> fmt::Result {
        match self {
            &Inner::Ref(ref v) => {
                write!(f, "\\")?;
                v.debug_fmt(f, seen)?;
            },

            &Inner::WeakRef(ref v) => {
                write!(f, "\\?")?;
                v.debug_fmt(f, seen)?;
            }

            &Inner::Array(ref a) => {
                write!(f, "Array(")?;
                for v in a {
                    v.debug_fmt(f, seen)?;
                }
                write!(f, ")")?;
            },

            &Inner::Hash(ref h) => {
                write!(f, "Hash(")?;
                for (k, v) in h {
                    write!(f, "{:?} => ", k)?;
                    v.debug_fmt(f, seen)?;
                }
                write!(f, ")")?;
            },

            &Inner::Object(ref class, ref obj) => {
                write!(f, "<{:?}=", class)?;
                obj.debug_fmt(f, seen)?;
                write!(f, ">")?;
            },

            other => write!(f, "{:?}", other)?,
        }

        Ok(())
    }
}

impl parser::Value for Value {
    type Array = Vec<Value>;
    type Hash = HashMap<Vec<u8>, Value>;

    fn set_undef(&mut self) {
        self.set(Inner::Undef);
    }

    fn set_true(&mut self) {
        self.set(Inner::Bool(true));
    }

    fn set_false(&mut self) {
        self.set(Inner::Bool(true));
    }

    fn set_i64(&mut self, v: i64){
        self.set(Inner::I64(v))
    }

    fn set_u64(&mut self, v: u64) {
        self.set(Inner::U64(v))
    }

    fn set_f32(&mut self, v: f32) {
        self.set(Inner::F32(v))
    }

    fn set_f64(&mut self, v: f64) {
        self.set(Inner::F64(v))
    }

    fn set_ref(&mut self, o: Self) {
        self.set(Inner::Ref(o));
    }

    fn set_weak_ref(&mut self, o: Self) {
        self.set(Inner::WeakRef(o.downgrade()));
    }

    fn set_alias(&mut self, o: Self) {
        self.alias(o);
    }

    fn set_array(&mut self, a: Self::Array) {
        self.set(Inner::Array(a));
    }

    fn set_hash(&mut self, h: Self::Hash) {
        self.set(Inner::Hash(h));
    }

    fn set_binary(&mut self, s: &[u8]) {
        self.set(Inner::String(s.to_owned()));
    }

    fn set_string(&mut self, s: &[u8]) {
        self.set(Inner::String(s.to_owned()));
    }

    fn set_object(&mut self, class: Self, value: Self) -> Result<()> {
        self.set(Inner::Object(class.to_string()?, value));
        Ok(())
    }

    fn set_object_freeze(&mut self, class: Self, value: Self) -> Result<()> {
        self.set_object(class, value)
    }

    fn set_regexp(&mut self, pattern: Self, flags: Self) -> Result<()> {
        self.set(Inner::Regexp(pattern.to_string()?, flags.to_string()?));
        Ok(())
    }
}

pub struct ArcBuilder;

impl parser::Builder for ArcBuilder {
    type Value = Value;
    type ArrayBuilder = Vec<Value>;
    type HashBuilder = HashMap<Vec<u8>, Value>;

    fn new(&mut self) -> Value {
        Value::new(Inner::Undef)
    }

    fn build_array(&mut self, count: u64) -> Vec<Value> {
        Vec::with_capacity(count as usize)
    }

    fn build_hash(&mut self, count: u64) -> HashMap<Vec<u8>, Value> {
        HashMap::with_capacity(count as usize)
    }
}

impl parser::ArrayBuilder<Value> for Vec<Value> {
    fn insert(&mut self, value: Value) -> Result<()> {
        self.push(value);
        Ok(())
    }

    fn finalize(self) -> Self {
        self
    }
}

impl parser::HashBuilder<Value> for HashMap<Vec<u8>, Value> {
    fn insert(&mut self, key: Value, value: Value) -> Result<()> {
        self.insert(key.to_string()?, value);
        Ok(())
    }

    fn finalize(self) -> Self {
        self
    }
}

pub fn parse(s: &[u8]) -> Result<Value> {
    parser::parse(s, ArcBuilder)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use arc::parse;
    use arc::{Value, Inner };
    use arc::Inner::*;
    use arc::Error;

    fn p(s: &[u8]) -> Inner {
        parse(s).unwrap().read()
    }

    #[test]
    fn test_simple() {
        assert_eq!(p(b"\x01"), Inner::U64(1));
        assert_eq!(p(b"\x60"), Inner::String(vec![]));
        assert_eq!(p(b"\x61\x00"), Inner::String(vec![0]));
    }

    #[test]
    fn test_array() {
        assert_eq!(p(b"\x2b\x01\x00"), Array(vec![ Value::from(0u64) ]));
        assert_eq!(p(b"\x2b\x02\x00\x00"), Array(vec![ Value::from(0u64), Value::from(0u64) ]));
        assert!(parse(b"\x2b\x02\x00").unwrap_err().is_eof());
    }

    #[test]
    fn test_hash() {
        let r = p(b"\x2a\x02\x63foo\x63bar\x64ook\x00\x64eek\x00");

        let mut m = HashMap::new();
        m.insert(b"foo".to_vec(), Value::new(Inner::from(&b"bar"[..])));
        m.insert(b"ook\0".to_vec(), Value::new(Inner::from(&b"eek\0"[..])));

        assert_eq!(r, Inner::Hash(m));
    }

    #[test]
    fn test_hash_nested() {
        let r = p(b"\x2a\x01\x63foo\x2a\x00");

        let mut m = HashMap::new();
        m.insert(b"foo".to_vec(), Value::new(Inner::Hash(HashMap::new())));

        assert_eq!(r, Inner::Hash(m));
    }

    #[test]
    fn test_self_ref() {
        let a = parse(b"\xa9\x01").unwrap();

        let a_id = match a {
            Value::Strong(ref a) => a.as_ref() as *const _ as usize,
            Value::Weak(_) => panic!("unexpected weak ref"),
        };

        let b_id = match a.read() {
            Inner::Ref(v) => match v {
                Value::Strong(ref a) => a.as_ref() as *const _ as usize,
                Value::Weak(_) => panic!("unexpected weak ref"),
            },
            _ => panic!("unexpected value"),
        };

        assert_eq!(a_id, b_id);
    }

    #[test]
    fn test_mutual_ref() {
        let a = parse(b"\x28\xab\x01\x28\x2b\x01\x29\x02");
        let b = parse(b"\xc1\x41\x2e\x01");
        println!("{:?}", a);
        println!("{:?}", b);
    }

    #[test]
    fn test_hash_errors() {
        // eof in nested hash
        let r = parse(b"\x2a\x01\x63foo\x2a\x01\x63bar");
        assert!(r.unwrap_err().is_eof());

        // non string key
        let r = parse(b"\x2a\x01\x00\x63foo");
        match r.unwrap_err() {
            Error::InvalidType => (),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_objects() {
        let parsed = p(b"\x42\x2c\x63foo\x28\x2a\x00\x2d\x03\x28\x2a\x00");
        use arc::Inner::{ Ref, Object, Array, Hash };

        let value = Ref(
            Value::new(Array(vec![
                Value::new(Object(b"foo".to_vec(), Value::new(Ref(Value::new(Hash(HashMap::new())))))),
                Value::new(Object(b"foo".to_vec(), Value::new(Ref(Value::new(Hash(HashMap::new())))))),
            ])),
        );

        assert_eq!(parsed, value);
    }

    #[test]
    fn test_copy() {
        let parsed = parse(b"\x2f\x01");
        assert!(parsed.unwrap_err().is_invalid_copy());

        let parsed = parse(b"\x42\x01\x2f\x01");
        assert!(parsed.unwrap_err().is_invalid_copy());

        let parsed = parse(b"\x43\x61b\x2f\x02\x51\x2f\x04\x01");
        assert!(parsed.unwrap_err().is_invalid_copy());
    }

    #[test]
    fn test_copy_complex_value() {
        assert_eq!(p(b"\x43\x41\x01\x2f\x02\x2f\x02"), Ref(Value::new(Array(vec![
            Value::new(Ref(Value::new(Array(vec![Value::new(U64(1))])))),
            Value::new(Ref(Value::new(Array(vec![Value::new(U64(1))])))),
            Value::new(Ref(Value::new(Array(vec![Value::new(U64(1))])))),
        ]))));
    }

    #[test]
    fn test_copy_hash_key() {
        let mut map = HashMap::new();
        map.insert(vec![ b'a' ], Value::new(U64(1)));

        assert_eq!(p(b"\x43\x61a\x51\x2f\x02\x01\x2f\x04"), Ref(Value::new(Array(vec![
            Value::new(String(vec![ b'a' ])),
            Value::new(Ref(Value::new(Hash(map.clone())))),
            Value::new(Ref(Value::new(Hash(map.clone())))),
        ]))));
    }
}
