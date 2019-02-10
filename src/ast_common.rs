// Data structures that are common between different program representations.
use std::fmt;

// Predicate names are tagged with whether they are system predicates or not.
pub enum Pred {
    Sys(String),
    User(String)
}

impl fmt::Display for Pred {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pred::Sys(ref p) => write!(f, "sys:{}", p),
            Pred::User(ref p) => write!(f, "{}", p)
        }
    }
}

// TODO: Implement string interning for atoms
pub type Atom = String;
