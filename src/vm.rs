use crate::ast_common::PredSig;
use crate::ir::*;
use crate::domains::*;
use crate::ast_common::Pred;
use crate::builtins::*;

pub struct VM {
    // we are using a linked list for now. if this becomes a bottleneck, we can switch to VecDeque but I suspect that will be beneficial considering that State is already a large data structure.
    pub state: Option<State>,
    pub program: Program,
    builtins: BuiltIns,
}

impl VM {
    pub fn new(program: Program) -> Self {
        VM {
            state: Self::singleton(State::new()),
            program: program,
            builtins: BuiltIns::new(),
        }
    }

    fn singleton(s: State) -> Option<State> {
        Some(s)
    }

    fn modify_then_pack<F: FnOnce(&mut State)>(mut s: State, f: F) -> Option<State> {
        f(&mut s);
        Self::singleton(s)
    }

    // process this state, potentially producing multiple states
    fn next(&self, mut s: State) -> Option<State> {
        use Insn::*;

        // advance the PC, we may do it on only the non-jump cases later on as an optimization perhaps but loading the checkpoint will dominate this probably anyway
        s.pc += 1;
        match &self.program.text[&s.local_state.predicate][s.pc] {
            PushValue(v) => Self::modify_then_pack(s, |s| s.local_state.push_value(v.clone())),
            Pop => Self::modify_then_pack(s, |s: &mut State| {s.local_state.op_stack.pop();}),
            Dup => Self::modify_then_pack(s, |s| s.local_state.dup()),
            Fresh => Self::modify_then_pack(s, |s| {
                let lv = s.fresh_lv();
                s.local_state.push_value(lv)
            }),
            Load(x) => Self::modify_then_pack(s, |s| s.local_state.load(x.clone())),
            Store(x) => Self::modify_then_pack(s, |s| s.local_state.store(x.clone())),
            Construct(f, n_args) => Self::modify_then_pack(s, |s| {
                let args = s.local_state.pop_n(n_args.clone());
                s.local_state.push_value(Value::Ctor(f.clone(), args));
            }),
            Unify => s.unify(),
            MkCheckpoint(label, offset) => panic!("not implemented"),
            Jump(offset) => {
                s.pc = (*offset as usize).wrapping_add(s.pc); // addition in 2's complement with no penalty
                Self::singleton(s)
            },
            Call(PredSig(Pred::User(pred), arity)) => {
                // TODO: error checking when loading the predicate
                s.call_user(pred, *arity);
                Self::singleton(s)
            },
            Call(PredSig(Pred::Sys(pred, arity), _)) => {
                if self.builtins.exists(pred, arity) {
                    let args = s.local_state.pop_n(*arity);
                    if self.builtins[&(pred.clone(), *arity)](args, &mut s) {
                        Self::singleton(s)
                    } else {
                        s.load_next_checkpoint()
                    }
                } else {
                    panic!("The built-in predicate {} does not exist", Pred::Sys(pred.clone(), *arity))
                }
            }
            Det(label) => panic!("not implemented"),
            DetUntil(label) => panic!("not implemented"),
            Fail => s.load_next_checkpoint(),
            Ret => s.ret(),
            Halt => Self::singleton(s), // halt and catch fire
        }
    }

    // make a small step
    pub fn step(&mut self) {
        if let Some(state) = self.state.take() {
            self.state = self.next(state);
        }
    }

    pub fn control(&self) -> Option<&Insn> {
        self.state.as_ref().map(|s| {
            &self.program.text[&s.local_state.predicate][s.pc]
        })
    }

    pub fn run(&mut self) {
        while self.state.is_some() && self.control() != Some(&Insn::Halt) {
            self.step();
        }
    }
}