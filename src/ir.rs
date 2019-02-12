// Intermediate representation that the virtual machine uses
use std::fmt;
use std::collections::HashMap;
use crate::ast_common::*;

// Representation for logic variables, subject to change
pub type LV = i64;

// Checkpoint labels are just enumerated ints.
pub type Label = i64;

// Instructions that the VM executes
pub enum Insn {
    PushValue(Value), // push given value
    Drop,
    Dup,
    Fresh, // push a fresh var on top of the stack
    Load(usize), // load from variable table
    Store(usize),
    Construct(Atom, usize),
    Unify,
    MkCheckpoint(Label, isize),
    Jump(isize),
    Call(Pred),
    Det(Label),
    DetUntil(Label),
    Fail,
    Ret,
    Halt
}

#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub enum Value {
    Atom(Atom),
    LV(LV),
    Num(i64),
    Ctor(Atom, Vec<Value>)
}

pub struct Program {
    text: HashMap<String, Vec<Insn>>, // code of each user predicate
}