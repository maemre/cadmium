// Data structures that are common between different program representations.
use std::fmt;

// Predicate names are tagged with whether they are system predicates or not.
#[derive(Hash,PartialOrd,Ord,PartialEq,Eq,Debug,Clone)]
pub enum Pred {
    Sys(String, usize),
    User(String)
}

impl fmt::Display for Pred {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pred::Sys(ref p, ref arity) => write!(f, "sys:{}/{}", p, arity),
            Pred::User(ref p) => write!(f, "{}", p)
        }
    }
}

// predicate signatures
#[derive(Eq,PartialEq,Hash,Debug,Clone)]
pub struct PredSig(pub Pred, pub usize);

impl fmt::Display for PredSig {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!("{}/{}", self.0, self.1))
    }
}

// TODO: Implement string interning for atoms
pub type Atom = String;
