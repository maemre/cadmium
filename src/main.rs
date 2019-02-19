extern crate nom;
extern crate im_rc;

pub mod ast_common;
pub mod ast;
pub mod ir;
pub mod parser;
pub mod vm;
pub mod domains;
pub mod unification;
pub mod ir_gen;
pub mod builtins;
use ast::*;

fn main() {
    println!("{}", Expr::Atom::<i64>("foo".to_string()));
}
