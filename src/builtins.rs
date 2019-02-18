// Implementations of built-in functions exposed as system predicates.

use crate::domains::*;
use crate::ir::*;
use std::collections::HashMap;
use std::ops::Index;

// An alias for a boxed function pointer to a built-in function implementation. With this definition, built-in functions are semi-deterministic.
// TODO: allow for nondeterministic built-ins in an efficient way
pub type BuiltInFn = Box<dyn Fn(Vec<Value>, &mut State) -> bool>;

// A struct containing mappings to all built-in functions to make calling them easy.
pub struct BuiltIns {
    impls: HashMap<(String, usize), BuiltInFn>
}

impl BuiltIns {
    pub fn new() -> Self {
        let mut impls: HashMap<(String, usize), BuiltInFn> = HashMap::new();
        // create the mapping for each built-in
        impls.insert(("print".to_string(), 1), Box::new(|args: Vec<Value>, _state| {
            print!("{}", args[0]);
            true
        }));

        BuiltIns { impls: impls }
    }

    pub fn exists(&self, name: &String, arity: &usize) -> bool {
        self.impls.contains_key(&(name.clone(), *arity))
    }
}

impl Index<&(String, usize)> for BuiltIns {
    type Output = BuiltInFn;

    fn index(&self, sig: &(String, usize)) -> &BuiltInFn {
        &self.impls[sig]
    }
}