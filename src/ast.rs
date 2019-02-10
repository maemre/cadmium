// AST for the front-end
use std::fmt;
use crate::ast_common;
use ast_common::*;

// String-based variable representation, the wrapper is there to enforce stricter
// type checking.
pub enum Var {
    Var(String)
}

// Programs are quantified over the variable representation. This allows us to
// use strings for variable names closer to front-end for certain AST
// manipulations such as underscore elimination, and use integers for variable
// names for a more efficient representation in later phases.
pub type Program<V> = Vec<PredDef<V>>;

// Predicate definition. `name` should always be a user predicate. TODO: Enforce
// this.
pub struct PredDef<V> {
    name: Pred,
    params: Expr<V>,
    body: Stmt<V>
}

pub enum Stmt<V> {
    And(Box<Stmt<V>>, Box<Stmt<V>>),
    Or(Box<Stmt<V>>, Box<Stmt<V>>),
    If(Box<Stmt<V>>, Box<Stmt<V>>),
    Unify(Expr<V>, Expr<V>),
    Call(Pred, Vec<Expr<V>>),
    Fail, // For convenience
    TrueS // For convenience
}

impl<V> fmt::Display for Stmt<V> where V: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Stmt::*;

        match self {
            And(ref s1, ref s2) => write!(f, "{}, {}", *s1, *s2),
            Or(ref s1, ref s2) => write!(f, "({}); ({})", *s1, *s2),
            If(ref s1, ref s2) => write!(f, "({}->{})", *s1, *s2),
            Unify(ref e1, ref e2) => write!(f, "{}={}", e1, e2),
            Call(ref p, ref args) =>
                match args.len() {
                    0 => write!(f, "{}", p),
                    _ => {
                        write!(f, "{}({}", p, args[0]);
                        for i in 1..args.len() {
                            write!(f, ", {}", args[i]);
                        }
                        write!(f, ")")
                    }
                }
            TrueS => write!(f, "true"),
            Fail => write!(f, "fail")
        }
    }
}

pub enum Expr<V> {
    Atom(Atom),
    PV(V),
    Num(i64),
    Ctor(Atom, Vec<Expr<V>>)
}


impl<V> fmt::Display for Expr<V> where V: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Expr::*;

        match self {
            Atom(ref a) => write!(f, "{}", a),
            PV(ref x) => write!(f, "{}", x),
            Num(ref n) => write!(f, "{}", n),
            Ctor(ref p, ref args) => {
                match args.len() {
                    0 => write!(f, "{}", p),
                    _ => {
                        write!(f, "{}({}", p, args[0]);
                        for i in 1..args.len() {
                            write!(f, ", {}", args[i]);
                        }
                        write!(f, ")")
                    }
                }
            }
        }
    }
}