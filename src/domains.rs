// Concrete domains for the VM
use crate::ir::*;
use crate::unification::*;
use std::collections::HashMap;

// State local to a frame in the call stack, except for control
#[derive(Debug)]
pub struct LocalState {
    // TODO: use a HashTrieMap for persistence, or even better just a Vec
    locals: HashMap<usize, Value>, // the environment
    opStack: Vec<Value>, // the operand stack, get rid of this?
    frameDepth: u32 // depth of this call stack frame, this is incremented on each call hence assigns a unique ID to each call. These are used for constructing checkpoint labels(?)
}

// The call stack, each frame consists of the PC and the local state.
pub type CallStack = Vec<(LocalState, usize)>;

#[derive(Debug)]
pub struct Checkpoint {
    label: (u32, Label), // label of the checkpoint, used for unrolling (Drop instruction)
    localState: LocalState,
    bindings: Unification<Value>, // the heap graph, as a Union-Find data structure
    pc: usize,
    callStack: CallStack
}

// Stack of check-points
pub type CPStack = Vec<Checkpoint>;

// State of the whole VM
pub struct State {
    localState: LocalState,
    cpStack: CPStack,
    pc: usize,
    callStack: CallStack,
    genIdx: usize, // a counter for new symbols, TODO: separate this to a global.
    unifyCount: usize // count #successful unifications for profiling. TODO: make this global.
}
