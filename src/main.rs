extern crate nom;
extern crate im_rc;

pub mod ast_common;
pub mod ast;
pub mod ir;
pub mod parser;
pub mod vm;
pub mod domains;
pub mod unification;
use ast::*;

fn main() {
    parser::var("foo");
    println!("{}", Expr::Atom::<i64>("foo".to_string()));
}
