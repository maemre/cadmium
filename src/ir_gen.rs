// IR-generating compiler. This compiler accepts only programs with numerical variables, i.e. `ast::Program<usize>`.

use crate::ir::*;
use crate::ast::*;
use crate::ast;
use crate::ir;
use crate::ast_common::*;
use std::collections::{HashMap, HashSet};

// The state of the IR-generating compiler. This structure carries information about the scope, the generated variable counter, etc.
pub struct IRGen {
    // program text of the generated IR so far
    ir_code: HashMap<PredSig, Vec<Insn>>,
    // counter for generated labels
    label_counter: Label,
    current_ir_code: Vec<Insn>,
}

impl IRGen {
    pub fn new() -> Self {
        IRGen { ir_code: HashMap::new(), label_counter: 0, current_ir_code: Vec::new() }
    }

    pub fn compile_program(&mut self, program: ast::Program<usize>) {
        for pred_def in program.into_iter() {
            self.compile_pred(pred_def);
        }
    }

    pub fn compile_pred(&mut self, pred_def: PredDef<usize>) {
        let sig = pred_def.sig();
        
        match pred_def.name {
            p@Pred::Sys(_, _) => panic!(format!("Trying to define the system predicate {} in user code!", p)),
            Pred::User(name) => {
                // we want each predicate to have only one definition by this point. Also, we don't allow re-definitions of system predicates. TODO: make these static checks
                assert!(!self.ir_code.contains_key(&sig), format!("Trying to redefine the predicate {} in user code!", name));
                // assert that we are not in the middle of compiling another predicate
                assert!(self.current_ir_code.is_empty(), "trying to compile a predicate while being in the middle of compiling another one");
                let arity = pred_def.params.len();
                // generate the IR that will unify the parameters with the arguments on stack
                self.compile_params(pred_def.params);
                // compile given statement
                self.compile_stmt(pred_def.body);
                // insert a halt instruction if we are working on main
                if name == "main" && arity  == 0 {
                    self.current_ir_code.push(Insn::Halt);
                }
                // Insert initialization code for all locals
                // TODO: do this after all optimizations and using a DFA to lower some unifications to Store instructions when one side is free and the other side is ground.
                
                let mut used_locals: HashSet<usize> = HashSet::new();
                for insn in self.current_ir_code.iter() {
                    if let Insn::Load(n) = insn {
                        used_locals.insert(*n);
                    }
                }

                let mut ir_code = Vec::with_capacity(used_locals.len() * 1 + self.current_ir_code.len());

                // generate the initialization code
                for n in used_locals.into_iter() {
                    ir_code.push(Insn::Fresh);
                    ir_code.push(Insn::Store(n));
                }

                // move the body we were working on to the initialization code
                ir_code.append(&mut self.current_ir_code);

                // insert the code for this predicate
                self.ir_code.insert(sig, ir_code);
            }
        }
    }

    // Generate the IR code that will unify the parameters with the values already on the stack
    pub fn compile_params(&mut self, params: Vec<Expr<usize>>) {
        for param in params.into_iter() {
            // push the expression to the op stack, then unify it with the argument that was on top of the stack already.
            self.compile_expr(param);
            self.current_ir_code.push(Insn::Unify);
        }
    }

    // generate the IR that will push the given expression to the op stack
    pub fn compile_expr(&mut self, expr: Expr<usize>) {
        use Expr::*;

        match expr {
            Atom(a) => self.current_ir_code.push(Insn::PushValue(Value::Atom(a))),
            PV(x) => self.current_ir_code.push(Insn::Load(x)),
            Num(n) => self.current_ir_code.push(Insn::PushValue(Value::Num(n))),
            Ctor(f, args) => {
                let n_args = args.len();
                // push the args to the stack
                for arg in args.into_iter() {
                    self.compile_expr(arg);
                }
                // construct the functor
                self.current_ir_code.push(Insn::Construct(f, n_args));
            }
        }
    }

    pub fn create_checkpoint(&mut self) {
        self.label_counter += 1;
        self.current_ir_code.push(Insn::MkCheckpoint(self.label_counter, 0));
    }

    // compile given statement and add it to the end of the current IR body
    pub fn compile_stmt(&mut self, stmt: Stmt<usize>) {
        use Stmt::*;

        match stmt {
            And(s1, s2) => {
                self.compile_stmt(*s1);
                self.compile_stmt(*s2);
            }
            Or(s1, s2) => {
                // we are compiling s1 ; s2 into
                // MkCheckpoint fresh_label, |[[s1]]| + 1
                // [[s1]]
                // Jump |[[s2]]|
                // [[s2]]

                // create the checkpoint
                let cp_pc = self.current_ir_code.len();
                self.create_checkpoint();
                // compile s1
                self.compile_stmt(*s1);
                // set checkpoint target
                let cp_target_offset = (self.current_ir_code.len() - cp_pc) as isize;
                self.current_ir_code[cp_pc].set_target(cp_target_offset);
                // create the jump instruction
                let jump_pc = self.current_ir_code.len();
                self.current_ir_code.push(Insn::Jump(0));
                // compile s2
                self.compile_stmt(*s2);
                // set jump target
                let jump_target_offset = (self.current_ir_code.len() - jump_pc) as isize;
                self.current_ir_code[jump_pc].set_target(jump_target_offset);
            }
            If(s1, s2, s3) => panic!("not implemented yet!"),
            Unify(e1, e2) => {
                self.compile_expr(e1);
                self.compile_expr(e2);
                self.current_ir_code.push(Insn::Unify);
            }
            Call(p, args) => {
                let arity = args.len();
                // push the arguments, right-to-left
                for expr in args.into_iter().rev() {
                    self.compile_expr(expr);
                }
                // call the predicate
                self.current_ir_code.push(Insn::Call(PredSig(p, arity)));
            }
            Fail => self.current_ir_code.push(Insn::Fail),
            True => {}
        }
    }

    // allow the user to inspect the generated code
    pub fn get_ir_ref(&self) -> &HashMap<PredSig, Vec<Insn>> {
        &self.ir_code
    }

    // extract the generated IR program and consume Self.
    pub fn get_ir_program(mut self) -> ir::Program {
        assert!(self.current_ir_code.is_empty(), "Tried to extract the program in middle of compiling a predicate");
        // Add halt at the end of main
        if let Some(main) = self.ir_code.get_mut(&PredSig(Pred::User("main".to_string()), 0)) {
            main.push(Insn::Halt);
        }

        ir::Program { text: self.ir_code }
    }
}