use std::mem;
use std::ops;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::clone::Clone;
use im_rc::HashMap;
use std::hash::Hash;
use std::cmp::Eq;
use crate::ir::*;

// Persistent version of Tarjan's union-find data structure. It is based on "A persistent union-find data structure" by Conchon et al.

#[derive(Clone)]
pub struct Unification where {
    parent: HashMap<Value, Value>
}

// TODO: implement path compression
impl Unification {
    pub fn new() -> Self {
        Unification {
            parent: HashMap::new()
        }
    }

    pub fn find<'a, 'b: 'a, 'c: 'a>(&'b self, x: &'c Value) -> &'a Value {
        match self.parent.get(x) {
            Some(y@Value::LV(_)) if x != y => self.find(y),
            _ => x
        }
    }

    // Unify given values, this clones the values into the union-find if they are not present.
    pub fn union(&self, x: &Value, y: &Value) -> Option<Self> {
        match (self.find(x), self.find(y)) {
            (x, y) if x == y => Some(self.clone()),
            (x@Value::LV(_), y) => {
                Some(Unification {
                    parent: self.parent.update(x.clone(), y.clone()).update(y.clone(), y.clone()),
                })
            }
            (x, y@Value::LV(_)) => self.union(x, y),
            (Value::Ctor(f, fArgs), Value::Ctor(g, gArgs)) if f == g && fArgs.len() == gArgs.len() => {
                (1..fArgs.len()).fold(Some(self.clone()), { |ufOption, i|
                    ufOption.and_then(|uf| uf.union(&fArgs[i], &gArgs[i]))
                })
            }
            _ => None // unification failure
        }
    }
}

impl fmt::Debug for Unification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        panic!("not implemented yet") // TODO: implement
    }
}