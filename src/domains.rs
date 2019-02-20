// Concrete domains for the VM
use crate::ast_common::*;
use crate::ir::*;
use crate::unification::*;
use im_rc::HashMap;


// State local to a frame in the call stack, except for control
// The operand stack is mutable and copied for checkpoints for now as it is mutated frequently.
#[derive(Debug)]
pub struct LocalState {
    // TODO: use a HashTrieMap for persistence, or even better just a Vec
    pub locals: HashMap<usize, Value>, // the environment
    pub op_stack: Vec<Value>, // the operand stack, get rid of this?
    pub predicate: PredSig,
    pub frame_depth: u32, // depth of this call stack frame, this is incremented on each call hence assigns a unique ID to each call. These are used for constructing checkpoint labels(?)
}

impl LocalState {
    pub fn new(predicate: PredSig, frame_depth: u32) -> Self {
        LocalState {
            locals: HashMap::new(),
            op_stack: vec![],
            frame_depth: frame_depth,
            predicate: predicate
        }
    }

    pub fn push_value(&mut self, v: Value) {
        self.op_stack.push(v);
    }

    pub fn dup(&mut self) {
        self.op_stack.push(self.op_stack.last().unwrap().clone())
    }

    pub fn load(&mut self, pv: usize) {
        self.op_stack.push(self.locals.get(&pv).unwrap().clone())
    }

    pub fn store(&mut self, pv: usize) {
        self.locals = self.locals.update(pv, self.op_stack.pop().unwrap())
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.op_stack.pop()
    }

    // pop N elements from the top of the operand stack
    pub fn pop_n(&mut self, n: usize) -> Vec<Value> {
        self.op_stack.split_off(self.op_stack.len() - n)
    }
}

// The call stack, each frame consists of the PC and the local state.
pub type CallStack = Vec<(LocalState, usize)>;

#[derive(Debug)]
pub struct Checkpoint {
    pub label: (u32, Label), // label of the checkpoint, used for unrolling (Drop instruction)
    pub local_state: LocalState,
    pub bindings: Unification, // the heap graph, as a Union-Find data structure
    pub pc: usize,
    pub call_stack: CallStack
}

// Stack of check-points
pub type CPStack = Vec<Checkpoint>;

// State of the whole VM
pub struct State {
    pub local_state: LocalState,
    pub bindings: Unification,
    pub cp_stack: CPStack,
    pub pc: usize,
    pub call_stack: CallStack,
    pub gen_idx: LV, // a counter for new symbols, TODO: separate this to a global.
    pub unify_count: usize // count #successful unifications for profiling. TODO: make this global.
}

impl State {
    pub fn new() -> Self {
        State {
            local_state: LocalState::new(PredSig(Pred::User("main".to_string()), 0), 0),
            bindings: Unification::new(),
            cp_stack: vec![],
            pc: 0,
            call_stack: vec![],
            gen_idx: 0,
            unify_count: 0
        }
    }

    pub fn fresh_lv(&mut self) -> Value {
        self.gen_idx += 1;
        Value::LV(self.gen_idx)
    }

    // return from predicate call while consuming this state
    pub fn ret(mut self) -> Option<State> {
        self.call_stack.pop().map(|(local_state, pc)| {
            self.local_state = local_state;
            self.pc = pc;
            self
        })
    }

    // perform unification of the top two stack values.
    pub fn unify(mut self) -> Option<Self> {
        if let Some((ref x, ref y)) = self.local_state.pop().and_then(|x| self.local_state.pop().map(|y| (x, y))) {
            if let Some(new_bindings) = self.bindings.union(x, y) {
                self.bindings = new_bindings;
                // increment # of successfull unifications
                self.unify_count += 1;
                Some(self)
            } else {
                self.load_next_checkpoint()
            }
        } else {
            panic!(format!("Program error at {}:{}. Not enough values to unify!", self.local_state.predicate, self.pc))
        }
    }

    // The current branch of execution failed, load the next checkpoint from the checkpoint stack
    pub fn load_next_checkpoint(mut self) -> Option<Self> {
        self.cp_stack.pop().map(|cp| self.load_checkpoint(cp))
    }

    // Consume this state and given checkpoint to load the checkpoint as the state
    pub fn load_checkpoint(mut self, cp: Checkpoint) -> Self {
        self.local_state = cp.local_state;
        self.bindings = cp.bindings;
        self.pc = cp.pc;
        self.call_stack = cp.call_stack;
        self
    }

    // Make a user predicate call, saves the local state and enters the predicate's body
    pub fn call_user(&mut self, pred: &str, argc: usize) {
        // load the new local state and extract the current one
        let new_frame_depth = self.local_state.frame_depth + 1;
        let last_frame = std::mem::replace(&mut self.local_state, LocalState::new(PredSig(Pred::User(pred.to_string()), argc), new_frame_depth));
        // save the return address
        self.call_stack.push((last_frame, self.pc));
        // move the PC to the beginning
        self.pc = 0;
    }
}