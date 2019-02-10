// Parser for the high-level AST

use ::nom::*;
use crate::ast_common::*;
use crate::ast::*;

named!(
    pub var<&str, String>,
    map_opt!(alphanumeric, |s: &str| {
        if s.chars().next().unwrap().is_uppercase() {
            Some(s.to_string())
        } else {
            None
        }
    })
);

named!(
    atom<&str, String>,
    map_opt!(alphanumeric, |s: &str| {
        if s.chars().next().unwrap().is_lowercase() {
            Some(s.to_string())
        } else {
            None
        }
    })
);

named!(
    ctor<&str, Expr<String>>,
    do_parse!(
        p: atom >>
        args: delimited!(tag!("("), many0!(expr), tag!(")")) >>
        (Expr::Ctor(p, args))
    )
);

named!(
    expr<&str, Expr<String>>,
    alt!(
        map!(var, |v| { Expr::PV::<String>(v) })
      | ctor | map!(atom, |a| { Expr::Atom(a.to_string()) } )
      | map_res!(digit::<&str>, |s| {
           i64::from_str_radix(s, 10).map(Expr::Num)
        })
      | delimited!(tag!("("), expr, tag!(")"))
    )
);

