use std::mem;
use std::ops;
use std::fmt;

// Imports from local data structures
use LinkedArray::*;

// Unification data structure with persistance and compression. It is based on "A persistent union-find data structure" by Conchon et al.

// Persistent arrays represented as either
//  - an ordinary array
//  - a delta to a linked persistent array
enum LinkedArray<T> {
    Arr(Vec<T>),
    Diff(usize, T, Box<LinkedArray<T>>) // Use Rc?
}

impl<T> LinkedArray<T> {
    // consume given vector and convert it into a persistent array
    fn new(v: Vec<T>) -> LinkedArray<T> {
        LinkedArray::Arr(v)
    }

    // return a new persistent array based on this one. The trick
    // here is to make the newest version of the array efficient while
    // accumulating changes in older versions which will be discarded
    // in case of not backtracking.
    // `self` is `&mut` because we are modifying it to have the difference.
    fn update(&mut self, i: usize, x: T) -> Self {
        let y = match self {
            Arr(v) => mem::replace(&mut v[i], x),
            Diff(_, _, _) => {
                // this is already a diff history, just append
                return Diff(i, x, Box::new(*self))
            }
        };

        mem::replace(self, Diff(i, y, Box::new(*self)))
    }
    
    // Baker's optimization from "Shallow binding makes functional arrays fast".
    // This function changes the direction of linking to make this version of the array base.
    fn reroot(&mut self) {
        if let Diff(i, x, a) = self {
            a.reroot(); // re-root the base of this array
            // the following will not fail because of contract of reroot
            if let Arr(v) = *a {
                let y = mem::replace(&mut v[i], x);
                mem::swap(self, Arr(v));
                *a = Diff(i, y, Box::new(self))
            }
        }
    }
}

impl<T> ops::Index<usize> for LinkedArray<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match self {
            Arr(v) => &v[i],
            Diff(j, x, a) => {
                if i == *j {
                    &x
                } else {
                    &a[i]
                }
            }
        }
    }
}

// TODO: add dynamic resizing
pub struct Unification<V> {
    rank: LinkedArray<V>,
    parent: LinkedArray<V>
}

impl<V> Unification<V> {
    fn new() -> Self {
        Unification {
            rank: LinkedArray::new(vec![]),
            parent: LinkedArray::new(vec![])
        }
    }

    fn union() {} // TODO: implement
}

impl<V: fmt::Debug> fmt::Debug for Unification<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        panic!("not implemented yet") // TODO: implement
    }
}