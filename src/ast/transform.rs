// The overall structure of the AST transformers

use std::collections::HashMap;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem;
use crate::ast::*;

// Types of AST transformers accepting ASTs with variable type Vin, producing ASTs with variable type Vout
pub trait Transformer<Vin, Vout> {
    // Transform input program by consuming it and self. self is consumed because transformers can be stateful
    fn transform(self, input: Program<Vin>) -> Program<Vout>;
}

// In-place transformer that uses in-place mutation as an optimization
pub trait InplaceTransformer<V> {
    fn transform_inplace(self, input: &mut Program<V>);
}

impl<V, T> Transformer<V, V> for T where T: InplaceTransformer<V> + Sized {
    fn transform(self, mut input: Program<V>) -> Program<V> {
        self.transform_inplace(&mut input);
        input
    }
}

// Transformer that consolidates all definitions of a predicate
pub struct ConsolidateDefs {
    defs: HashMap<(Pred, usize), Vec<(Vec<Expr<String>>, Stmt<String>)>>,
}

impl ConsolidateDefs {
    pub fn new() -> Self {
        ConsolidateDefs {
            defs: HashMap::new(),
        }
    }

    fn add_pred_def(&mut self, pred_def: PredDef<String>) {
        let name_with_arity = (pred_def.name, pred_def.params.len());
        let clause_def = (pred_def.params, pred_def.body);
        if let Some(ref mut clause_defs) = self.defs.get_mut(&name_with_arity) {
            clause_defs.push(clause_def);
        } else {
            self.defs.insert(name_with_arity, vec![clause_def]);
        }
    }
}

impl Transformer<String, String> for ConsolidateDefs {
    fn transform(mut self, program: Program<String>) -> Program<String> {
        use Stmt::*;

        for pred_def in program.into_iter() {
            self.add_pred_def(pred_def);
        }

        Vec::from_iter(self.defs.into_iter().map(|((pred, n_args), bodies)| {
            let params: Vec<Expr<String>> = Vec::from_iter((1..n_args).map(|i| Expr::PV(format!("_P{}", i))));

            let new_body = bodies.into_iter().fold(Fail, |acc, (clause_params, body)| {
                // create a statement that will assign each clause parameter to the corresponded generated predicate parameter
                let param_assignment = params.iter().zip(clause_params.into_iter()).fold(True, |acc, (p, cp)| { And(Box::new(acc), Box::new(Unify(p.clone(), cp))) });

                Or(Box::new(acc), Box::new(And(Box::new(param_assignment), Box::new(body))))
            });

            PredDef {
                name: pred,
                params: params,
                body: new_body
            }
        }))
    }
}

// Transformer that eliminates underscores in the program. Note: the variable names starting with _ are reserved for the compiler.
pub struct UnderscoreElim {
    underscore_counter: usize
}

impl UnderscoreElim {
    pub fn new() -> Self { UnderscoreElim { underscore_counter: 0 } }

    fn gen_var(&mut self) -> String {
        self.underscore_counter += 1;
        format!("_G{}", self.underscore_counter)
    }

    fn transform_pred(&mut self, pred_def: &mut PredDef<String>) {
        for e in pred_def.params.iter_mut() {
            self.transform_expr(e);
        }
        self.transform_stmt(&mut pred_def.body);
    }

    fn transform_stmt(&mut self, stmt: &mut Stmt<String>) {
        use Stmt::*;

        match stmt {
            And(ref mut s1, ref mut s2) => {
                self.transform_stmt(s1);
                self.transform_stmt(s2);
            }
            Or(ref mut s1, ref mut s2) => {
                self.transform_stmt(s1);
                self.transform_stmt(s2);
            }
            If(ref mut s1, ref mut s2, ref mut s3) => {
                self.transform_stmt(s1);
                self.transform_stmt(s2);
                self.transform_stmt(s3);
            }
            Unify(ref mut e1, ref mut e2) => {
                self.transform_expr(e1);
                self.transform_expr(e2);
            }
            Call(_, args) => {
                for arg in args.iter_mut() {
                    self.transform_expr(arg);
                }
            }
            Fail | True => {}
        }
    }

    fn transform_expr(&mut self, expr: &mut Expr<String>) {
        use Expr::*;

        match expr  {
            PV(ref mut x) if *x == "_" => {
                *x = self.gen_var();
            }
            Ctor(_, args) => {
                for arg in args.iter_mut() {
                    self.transform_expr(arg);
                }
            }
            PV(_) | Atom(_) | Num(_) => {}
        }
    }
}

impl InplaceTransformer<String> for UnderscoreElim {
    fn transform_inplace(mut self, input: &mut Program<String>) {
        for pred_def in input.iter_mut() {
            self.transform_pred(pred_def);
        }
    }
}

// Eliminate unnecessary `TrueS` and `Fail` statements. This pass is useful after some passes that produce TrueS/Fail when a statement with no side effect is needed.
pub struct IdempotentElim<V> {
    phantom_data: PhantomData<V>
}

// we need the PartialEq on V to derive it for statement equality
impl<V: PartialEq> IdempotentElim<V> {
    pub fn new() -> Self {
        IdempotentElim { phantom_data: PhantomData }
    }

    fn transform_stmt(&self, stmt: &mut Stmt<V>) {
        use Stmt::*;

        // transform the inner statements recursively
        stmt.traverse_mut(&mut |s| {
            // extract the statement to mutate it freely
            let mut temp = mem::replace(s, Fail);
            match temp {
                And(ref mut s1, ref mut s2) if  **s1 == True => 
                    mem::swap(s, s2.as_mut()),
                And(ref mut s1, ref mut s2) if  **s2 == True =>
                    mem::swap(s, s1.as_mut()),
                Or(ref mut s1, ref mut s2) if **s1 == Fail => { mem::swap(s, s2.as_mut()); }
                Or(ref mut s1, ref mut s2) if **s2 == Fail => { mem::swap(s, s1.as_mut()); }
                If(ref mut s1, _, ref mut s3) if **s1 == Fail => { mem::swap(s, s3.as_mut()); }
                If(ref mut s1, ref mut s2, _) if **s1 == True => { mem::swap(s, s2.as_mut()); }
                _ => mem::swap(s, &mut temp) // put the value back
            };
        });
    }
}

impl<V: PartialEq> InplaceTransformer<V> for IdempotentElim<V> {
    fn transform_inplace(self, input: &mut Program<V>) {
        for pred_def in input.iter_mut() {
            self.transform_stmt(&mut pred_def.body);
        }
    }
}