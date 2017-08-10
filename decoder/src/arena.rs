use std;
use std::cell::Cell;
use std::collections::HashMap;

use typed_arena;

use parser;
pub use parser::Error;
pub use parser::Result;

pub struct Arena<'a> {
    values: typed_arena::Arena<Cell<Inner<'a>>>,
    arrays: typed_arena::Arena<Vec<Value<'a>>>,
    hashes: typed_arena::Arena<HashMap<&'a str, Value<'a>>>,
}

impl<'a> Arena<'a> {
    pub fn new() -> Self {
        Arena {
            values: typed_arena::Arena::new(),
            arrays: typed_arena::Arena::new(),
            hashes: typed_arena::Arena::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Inner<'a> {
    Undef,
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(&'a [u8]),
    Ref(Value<'a>),
    WeakRef(Value<'a>),
    Array(&'a [Value<'a>]),
    Hash(&'a HashMap<&'a str, Value<'a>>),
    Object(Value<'a>, Value<'a>),
    Bool(bool),
    Regexp(Value<'a>, Value<'a>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Value<'a: 'a>(pub &'a Cell<Inner<'a>>);

impl<'a> parser::Value<'a> for Value<'a> {
    type Array = &'a [Value<'a>];
    type Hash = &'a HashMap<&'a str, Value<'a>>;

    fn set_undef(&mut self) {
        self.set(Inner::Undef);
    }

    fn set_true(&mut self) {
        self.set(Inner::Bool(true));
    }

    fn set_false(&mut self) {
        self.set(Inner::Bool(true));
    }

    fn set_i64(&mut self, v: i64) {
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

    fn set_binary(&mut self, s: &'a [u8]) {
        self.set(Inner::String(s));
    }

    fn set_string(&mut self, s: &'a [u8]) {
        self.set(Inner::String(s));
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
        self.0.set(inner)
    }
}

pub struct ArenaBuilder<'a: 'a> {
    arena: &'a Arena<'a>,
}

impl<'a> ArenaBuilder<'a> {
    pub fn new(arena: &'a Arena<'a>) -> ArenaBuilder<'a> {
        ArenaBuilder { arena: arena }
    }
}

impl<'a> parser::Builder<'a> for ArenaBuilder<'a> {
    type Value = Value<'a>;
    type ArrayBuilder = &'a mut Vec<Value<'a>>;
    type HashBuilder = &'a mut HashMap<&'a str, Value<'a>>;

    fn new(&mut self) -> Value<'a> {
        Value(self.arena.values.alloc(Cell::new(Inner::Undef)))
    }

    fn build_array(&mut self, count: u64) -> &'a mut Vec<Value<'a>> {
        self.arena.arrays.alloc(Vec::with_capacity(count as usize))
    }

    fn build_hash(&mut self, count: u64) -> &'a mut HashMap<&'a str, Value<'a>> {
        self.arena.hashes.alloc(
            HashMap::with_capacity(count as usize),
        )
    }
}

impl<'a> parser::ArrayBuilder<'a, Value<'a>> for &'a mut Vec<Value<'a>> {
    fn insert(&mut self, v: Value<'a>) -> Result<()> {
        (*self).push(v);
        Ok(())
    }

    fn finalize(self) -> &'a [Value<'a>] {
        self
    }
}

impl<'a> parser::HashBuilder<'a, Value<'a>> for &'a mut HashMap<&'a str, Value<'a>> {
    fn insert(&mut self, key: &'a [u8], value: Value<'a>) -> Result<()> {
        let s = match std::str::from_utf8(key) {
            Ok(s) => s,
            _ => return Err(Error::InvalidType),
        };

        (*self).insert(s, value);

        Ok(())
    }

    fn finalize(self) -> &'a HashMap<&'a str, Value<'a>> {
        self
    }
}

pub fn parse<'a>(s: &'a [u8], arena: &'a Arena<'a>) -> Result<Value<'a>> {
    let builder = ArenaBuilder { arena: arena };
    parser::parse(s, builder)
}

#[cfg(test)]
mod test {
    use arena::Arena;
    use arena::Value;
    use arena::Inner;
    use arena::parse;


    #[test]
    fn test_self_ref() {
        let arena = Arena::new();
        let a = parse(b"\xa9\x01", &arena).unwrap();
        let a_id = a.0 as *const _ as usize;
        let b_id = match a.0.get() {
            Inner::Ref(Value(b)) => b as *const _ as usize,
            _ => panic!("expecting reference"),
        };
        assert_eq!(a_id, b_id);
    }
}
