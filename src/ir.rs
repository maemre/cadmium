// Intermediate representation that the virtual machine uses
use std::collections::HashMap;
use std::fmt;
use crate::ast_common::*;

// Representation for logic variables, subject to change
pub type LV = i64;

// Checkpoint labels are just enumerated ints.
pub type Label = i64;

// Instructions that the VM executes
#[derive(PartialEq,Eq,Debug)]
pub enum Insn {
    PushValue(Value), // push given value
    Pop,
    Dup,
    Fresh, // push a fresh var on top of the stack
    Load(usize), // load from variable table
    Store(usize),
    Construct(Atom, usize),
    Unify, // unify the top 2 values on the stack then remove them
    MkCheckpoint(Label, isize),
    Jump(isize),
    Call(PredSig),
    Det(Label),
    DetUntil(Label),
    Fail,
    Ret,
    Halt
}

impl Insn {
    // Set the target offset if this instruction is a checkpoint instruction or a jump instruction
    pub fn set_target(&mut self, target: isize) {
        if let Insn::MkCheckpoint(_, ref mut target_of_self) = self {
            *target_of_self = target;
        } else if let Insn::Jump(ref mut target_of_self) = self {
            *target_of_self = target;
        } else {
            panic!("Tried to set the target of a non-jump, non-checkpoint instruction")
        }
    }
}

// TODO: use bigints for arithmetic for a more fair comparison with existing Prolog engines
#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub enum Value {
    Atom(Atom),
    LV(LV),
    Num(i64),
    Ctor(Atom, Vec<Value>)
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use Value::*;

        match self {
            Atom(a) => formatter.write_str(&a),
            LV(x) => formatter.write_fmt(format_args!("_LV{}", x)),
            Num(n) => formatter.write_fmt(format_args!("{}", n)),
            Ctor(f, args) => {
                for arg in args.iter() {
                    arg.fmt(formatter)?;
                }
                formatter.write_str(&f)
            }
        }
    }
}

pub struct Program {
    pub text: HashMap<PredSig, Vec<Insn>>, // code of each user predicate
}