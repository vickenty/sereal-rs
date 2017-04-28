use std;
use std::cell::Cell;
use std::ptr;
use std::mem;
use std::collections::HashMap;

use typed_arena;

use parser;
pub use parser::Error;
pub use parser::Result;

pub struct Arena<'a, 'buf: 'a> {
    values: typed_arena::Arena<Cell<Inner<'a, 'buf>>>,
    arrays: typed_arena::Arena<Value<'a, 'buf>>,
    hashes: typed_arena::Arena<HashMap<&'buf str, Value<'a, 'buf>>>,
}

impl<'a, 'buf> Arena<'a, 'buf> {
    pub fn new() -> Self {
        Arena {
            values: typed_arena::Arena::new(),
            arrays: typed_arena::Arena::new(),
            hashes: typed_arena::Arena::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Inner<'a, 'buf: 'a> {
    Undef,
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(&'buf [u8]),
    Ref(Value<'a, 'buf>),
    WeakRef(Value<'a, 'buf>),
    Array(&'a [Value<'a, 'buf>]),
    Hash(&'a HashMap<&'buf str, Value<'a, 'buf>>),
    Object(Value<'a, 'buf>, Value<'a, 'buf>),
    Bool(bool),
    Regexp(Value<'a, 'buf>, Value<'a, 'buf>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Value<'a, 'buf: 'a>(pub &'a Cell<Inner<'a, 'buf>>);

impl<'a, 'buf> parser::Value<'buf> for Value<'a, 'buf> {
    type Array = &'a [Value<'a, 'buf>];
    type Hash = &'a HashMap<&'buf str, Value<'a, 'buf>>;

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

    fn set_binary(&mut self, s: &'buf [u8]) {
        self.set(Inner::String(s));
    }

    fn set_string(&mut self, s: &'buf [u8]) {
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

impl<'a, 'buf> Value<'a, 'buf> {
    fn set(&self, inner: Inner<'a, 'buf>) {
        self.0.set(inner)
    }
}

pub struct ArenaBuilder<'a, 'buf: 'a> {
    arena: &'a Arena<'a, 'buf>,
}

impl<'a, 'buf> ArenaBuilder<'a, 'buf> {
    pub fn new(arena: &'a Arena<'a, 'buf>) -> ArenaBuilder<'a, 'buf> {
        ArenaBuilder { arena: arena }
    }
}

impl<'a, 'buf> parser::Builder<'buf> for ArenaBuilder<'a, 'buf> {
    type Value = Value<'a, 'buf>;
    type ArrayBuilder = ArrayBuilder<'a, 'buf>;
    type HashBuilder = &'a mut HashMap<&'buf str, Value<'a, 'buf>>;

    fn new(&mut self) -> Value<'a, 'buf> {
        Value(self.arena.values.alloc(Cell::new(Inner::Undef)))
    }

    fn build_array(&mut self, count: u64) -> ArrayBuilder<'a, 'buf> {
        ArrayBuilder::new(&self.arena.arrays, count)
    }

    fn build_hash(&mut self, count: u64) -> &'a mut HashMap<&'buf str, Value<'a, 'buf>> {
        self.arena.hashes.alloc(HashMap::with_capacity(count as usize))
    }
}

pub struct ArrayBuilder<'a, 'buf: 'a> {
    base: *mut [Value<'a, 'buf>],
    next: *mut Value<'a, 'buf>,
    rest: usize,
}

impl<'a, 'buf> ArrayBuilder<'a, 'buf> {
    fn new(arena: &'a typed_arena::Arena<Value<'a, 'buf>>, cap: u64) -> Self {
        let cap = cap as usize;
        unsafe {
            let slice = arena.alloc_uninitialized(cap);
            let first = &mut (*slice)[0] as *mut _;
            ArrayBuilder {
                base: slice,
                next: first,
                rest: cap,
            }
        }
    }
}

impl<'a, 'buf> parser::ArrayBuilder<'buf, Value<'a, 'buf>> for ArrayBuilder<'a, 'buf> {
    fn insert(&mut self, v: Value<'a, 'buf>) -> Result<()> {
        assert!(self.rest > 0);
        self.rest -= 1;
        unsafe {
            ptr::write(self.next, v);
            self.next = self.next.offset(1);
        }
        Ok(())
    }

    fn finalize(self) -> &'a [Value<'a, 'buf>] {
        assert!(self.rest == 0);
        unsafe { mem::transmute(self.base) }
    }
}

impl<'a, 'buf> parser::HashBuilder<'buf, Value<'a, 'buf>> for &'a mut HashMap<&'buf str, Value<'a, 'buf>> {
    fn insert(&mut self, key: Value<'a, 'buf>, value: Value<'a, 'buf>) -> Result<()> {
        let s = match key.0.get() {
            Inner::String(s) => match std::str::from_utf8(s) {
                Ok(s) => s,
                _ => return Err(Error::InvalidType),
            },
            _ => return Err(Error::InvalidType),
        };

        (*self).insert(s, value);

        Ok(())
    }

    fn finalize(self) -> &'a HashMap<&'buf str, Value<'a, 'buf>> {
        self
    }
}

pub fn parse<'a, 'buf>(s: &'buf [u8], arena: &'a Arena<'a, 'buf>) -> Result<Value<'a, 'buf>> {
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
