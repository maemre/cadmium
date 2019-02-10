#[macro_use]
extern crate nom;

pub mod ast_common;
pub mod ast;
pub mod ir;
pub mod parser;
use ast::*;

fn main() {
    parser::var("foo");
    println!("{}", Expr::Atom::<i64>("foo".to_string()));
}
