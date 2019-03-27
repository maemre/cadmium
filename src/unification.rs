use crate::ir::*;
use im_rc::HashMap;
use std::clone::Clone;
use std::fmt;

// Persistent version of Tarjan's union-find data structure. It is based on "A persistent union-find data structure" by Conchon et al.

#[derive(Clone)]
pub struct Unification {
    parent: HashMap<Value, Value>,
}

// TODO: implement path compression
impl Unification {
    pub fn new() -> Self {
        Unification {
            parent: HashMap::new(),
        }
    }

    pub fn find<'a, 'b: 'a, 'c: 'a>(&'b self, x: &'c Value) -> &'a Value {
        match self.parent.get(x) {
            Some(y @ Value::LV(_)) if x != y => self.find(y),
            Some(y) => y,
            None => x,
        }
    }

    // Unify given values, this clones the values into the union-find if they are not present.
    pub fn union(&self, x: &Value, y: &Value) -> Option<Self> {
        match (self.find(x), self.find(y)) {
            (x, y) if x == y => Some(self.clone()),
            (x @ Value::LV(_), y) => Some(Unification {
                parent: self
                    .parent
                    .update(x.clone(), y.clone())
                    .update(y.clone(), y.clone()),
            }),
            (x, y @ Value::LV(_)) => self.union(y, x),
            (Value::Ctor(f, f_args), Value::Ctor(g, g_args))
                if f == g && f_args.len() == g_args.len() =>
            {
                (1..f_args.len()).fold(Some(self.clone()), {
                    |maybe_uf, i| maybe_uf.and_then(|uf| uf.union(&f_args[i], &g_args[i]))
                })
            }
            _ => None, // unification failure
        }
    }
}

impl fmt::Debug for Unification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.parent.fmt(f)
    }
}
