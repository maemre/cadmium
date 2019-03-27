extern crate nom;
extern crate im_rc;
extern crate rustyline;

pub mod ast_common;
pub mod ast;
pub mod ir;
pub mod parser;
pub mod vm;
pub mod domains;
pub mod unification;
pub mod ir_gen;
pub mod builtins;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use nom::types::CompleteStr;

use std::collections::HashMap;
use ast::*;
use ast::transform::*;
use vm::VM;
use ir_gen::IRGen;

// Compile given top-level statement to IR
fn compile_stmt(s: Stmt<String>) -> ir::Program {
    let input_ast = vec![PredDef::new("main", Vec::new(), s)];
    // The AST after eliminating multiple clauses, underscores, etc. Also, the variables are renamed into numbers.
    let lowered_ast = {
        IdempotentElim::new().transform(
            EnumerateVariables::new().transform(
                UnderscoreElim::new().transform(
                    ConsolidateDefs::new().transform(input_ast))))
    };
    let mut ig = IRGen::new();
    ig.compile_program(lowered_ast);
    ig.get_ir_program()
}

fn main() {
    let mut rl = Editor::<()>::new();

    if rl.load_history(".cadmium.hist").is_err() {
        println!("Creating history file.");
    }

    // Set up the compiler and the vm
    let mut vm = VM::new(ir::Program { text: HashMap::new() });
    let mut run_all = |stmts: Vec<Stmt<String>>| {
        for s in stmts.into_iter() {
            println!("running: {:?}", &s);
            let ir_code = compile_stmt(s);
            println!("IR code: {:?}", ir_code.text);
            vm = VM::new(ir_code);
            vm.run();
            if let Some(state) = &vm.state {
                for (x, v) in state.local_state.locals.iter() {
                    println!("{} = {}", x, state.bindings.find(v));
                }
                println!("bindings: {:?}", state.bindings);
            }
        }
    };

    // unused parts of the previous line
    let mut previous = "".to_string();

    // the repl
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(mut line) => {
                if &previous != "" {
                    previous.push_str(&line);
                } else {
                    std::mem::swap(&mut previous, &mut line);
                }

                match parser::top_level(CompleteStr(&previous)) {
                    Ok((CompleteStr(rest), stmts)) => {
                        println!("parsed: {:?}", stmts);
                        run_all(stmts);
                        previous = rest.trim().to_string();
                    }
                    result => {
                        println!("Parse error: {:?}", result);
                        previous.clear();
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("readline error: {:?}", err);
            }
        }
    }
}
