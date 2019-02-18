// Parser for the high-level AST

use ::nom::*;
use crate::ast_common::*;
use crate::ast::*;

named!(
    var_or_atom<&str, &str>,
    take_while1!(|c: char| {
        c.is_alphanumeric() || c == '_'
    })
);

named!(
    pub var<&str, String>,
    map_opt!(var_or_atom, |s: &str| {
        if s.chars().next().unwrap().is_uppercase() {
            Some(s.to_string())
        } else if s.starts_with("_") && !s.starts_with("__") {
            Some(s.to_string())
        } else {
            None
        }
    })
);

named!(
    pub atom<&str, String>,
    map_opt!(var_or_atom, |s: &str| {
        if s.chars().next().unwrap().is_lowercase() {
            Some(s.to_string())
        } else {
            None
        }
    })
);

named!(unum<&str, usize>, map_res!(digit1::<&str>, |s| {
           usize::from_str_radix(s, 10)
        }));

named!(num<&str, i64>, map_res!(digit1::<&str>, |s| {
           i64::from_str_radix(s, 10)
        }));

named!(
    pub pred<&str, Pred>,
    alt!(
        do_parse!(
            tag!("sys:") >>
            p: atom >>
            tag!("/") >>
            arity: unum >>
            (Pred::Sys(p, arity)))
      | map!(atom, |a| Pred::User(a))
    )
);

named!(
    ctor<&str, Expr<String>>,
    do_parse!(
        p: atom >>
        args: delimited!(tag!("("), many0!(expr), tag!(")")) >>
        (Expr::Ctor(p, args))
    )
);

named!(
    pub expr<&str, Expr<String>>,
    alt!(
        map!(var, |v| { Expr::PV::<String>(v) })
      | ctor | map!(atom, |a| { Expr::Atom(a.to_string()) } )
      | map!(num, Expr::Num)
      | delimited!(tag!("("), expr, tag!(")"))
    )
);

// A simple operator parser with hardcoded precedence
named!(conjunct<&str, Stmt<String>>,
    alt!(
        do_parse!(
            e1: expr >>
            tag!("=") >>
            e2: expr >>
            (Stmt::Unify(e1, e2))
        )
      | do_parse!(
            p: pred >>
            args: delimited!(tag!("("), separated_list!(tag!(","), expr), tag!(")")) >>
            (Stmt::Call(p, args))
      )
      | delimited!(tag!("("), stmt, tag!(")"))
    )
);

named!(
    disjunct<&str, Stmt<String>>,
    alt!(
        do_parse!(
            s1: conjunct >>
            tag!(",") >>
            s2: conjunct >>
            (Stmt::And(Box::new(s1), Box::new(s2)))
        )
    )
);

named!(
    pub stmt<&str, Stmt<String>>,
    alt!(
        do_parse!(
            s1: disjunct >>
            tag!("->") >>
            s2: disjunct >>
            tag!(";") >>
            s3: disjunct >>
            (Stmt::If(Box::new(s1), Box::new(s2), Box::new(s3)))
        )
    )
);

named!(
    pub pred_def<&str, PredDef<String>>,
    do_parse!(
        name: atom >>
        params: delimited!(tag!("("), separated_list!(tag!(","), expr), tag!(")")) >>
        body: map!(opt!(preceded!(tag!(":-"), stmt)),
            |b| { b.unwrap_or(Stmt::True) }
        ) >>
        tag!(".") >>
        (PredDef {
            name: Pred::User(name),
            params: params,
            body: body
        })
    )
);

named!(
    pub program<&str, Program<String>>,
    exact!(many1!(pred_def))
);

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use Expr::*;
    use Stmt::*;

    #[test]
    fn test_var() {
        assert_eq!(var("A "), Ok((" ", "A".to_string())));
        assert_eq!(var("AbCdAA "), Ok((" ", "AbCdAA".to_string())));
        assert_eq!(var("A b"), Ok((" b", "A".to_string())));
        assert_eq!(var("_ "), Ok((" ", "_".to_string())));
        assert_eq!(var("_X "), Ok((" ", "_X".to_string())));
        if let Err(nom::Err::Error(_)) = var("__ ") {
        } else {
            assert!(false, "variables starting with two underscores should be rejected")
        }
    }

    #[test]
    fn test_atom() {
        assert_eq!(atom("a "), Ok((" ", "a".to_string())));
        assert_eq!(atom("ab__C_dAA "), Ok((" ", "ab__C_dAA".to_string())));
        assert_eq!(atom("a b"), Ok((" b", "a".to_string())));
        if let Err(nom::Err::Error(_)) = atom("_ ") {
        } else {
            assert!(false, "variables starting with two underscores should be rejected")
        }
    }

    #[test]
    fn test_pred() {
        assert_eq!(pred("foo "), Ok((" ", Pred::User("foo".to_string()))));
        assert_eq!(pred("sys:foo/2 "), Ok((" ", Pred::Sys("foo".to_string(), 2))));

        if let Err(nom::Err::Error(_)) = pred("sys:foo/-1 ") {
        } else {
            assert!(false, "system predicates with negative arity should be rejected")
        }
    }
}