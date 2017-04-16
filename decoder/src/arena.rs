use std::cell::RefCell;
use typed_arena::Arena;

use parser;
pub use parser::Error;
pub use parser::Result;

#[derive(Clone, Debug, PartialEq)]
pub enum Inner<'a> {
    Undef,
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(Vec<u8>),
    Ref(Value<'a>),
    WeakRef(Value<'a>),
    Array(Vec<Value<'a>>),
    Hash(Vec<(Value<'a>, Value<'a>)>),
    Object(Value<'a>, Value<'a>),
    Bool(bool),
    Regexp(Value<'a>, Value<'a>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Value<'a>(&'a RefCell<Inner<'a>>);

impl<'a> parser::Value for Value<'a> {
    type Array = Vec<Value<'a>>;
    type Hash = Vec<(Value<'a>, Value<'a>)>;

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
        self.set(Inner::WeakRef(o));
    }

    fn set_alias(&mut self, o: Self) {
        self.0 = o.0;
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
        self.set(Inner::Object(class, value));
        Ok(())
    }

    fn set_object_freeze(&mut self, class: Self, value: Self) -> Result<()> {
        self.set_object(class, value)
    }

    fn set_regexp(&mut self, pattern: Self, flags: Self) -> Result<()> {
        self.set(Inner::Regexp(pattern, flags));
        Ok(())
    }
}

impl<'a> Value<'a> {
    fn set(&self, inner: Inner<'a>) {
        *self.0.borrow_mut() = inner;
    }
}

pub struct ArenaBuilder<'a> {
    arena: &'a Arena<RefCell<Inner<'a>>>,
}

impl<'a> parser::Builder for ArenaBuilder<'a> {
    type Value = Value<'a>;
    type ArrayBuilder = Vec<Value<'a>>;
    type HashBuilder = Vec<(Value<'a>, Value<'a>)>;

    fn new(&mut self) -> Value<'a> {
        Value(self.arena.alloc(RefCell::new(Inner::Undef)))
    }

    fn build_array(&mut self, count: u64) -> Vec<Value<'a>> {
        Vec::with_capacity(count as usize)
    }

    fn build_hash(&mut self, count: u64) -> Vec<(Value<'a>, Value<'a>)> {
        Vec::with_capacity(count as usize)
    }
}

impl<'a> parser::ArrayBuilder<Value<'a>> for Vec<Value<'a>> {
    fn insert(&mut self, v: Value<'a>) -> Result<()> {
        self.push(v);
        Ok(())
    }

    fn finalize(self) -> Vec<Value<'a>> {
        self
    }
}

impl<'a> parser::HashBuilder<Value<'a>> for Vec<(Value<'a>, Value<'a>)> {
    fn insert(&mut self, key: Value<'a>, value: Value<'a>) -> Result<()> {
        self.push((key, value));
        Ok(())
    }

    fn finalize(self) -> Vec<(Value<'a>, Value<'a>)> {
        self
    }
}

pub fn parse<'a>(s: &[u8], arena: &'a Arena<RefCell<Inner<'a>>>) -> Result<Value<'a>> {
    let builder = ArenaBuilder { arena: arena };
    parser::parse(s, builder)
}

#[cfg(test)]
mod test {
    use arena::Inner;
    use arena::Value;
    use arena::parse;
    use typed_arena::Arena;

    #[test]
    fn test_self_ref() {
        let arena = Arena::new();
        let a = parse(b"\xa9\x01", &arena).unwrap();
        let a_id = a.0 as *const _ as usize;
        let b_id = match &*a.0.borrow() {
            &Inner::Ref(Value(b)) => b as *const _ as usize,
            _ => panic!("expecting reference"),
        };
        assert_eq!(a_id, b_id);
    }
}