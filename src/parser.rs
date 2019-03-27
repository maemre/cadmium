// Parser for the high-level AST

use ::nom::*;
use ::nom::types::CompleteStr;
use crate::ast_common::*;
use crate::ast::*;

// Use CompleteStr to communicate with nom that we have the complete inputs.

named!(
    alnum_or_underscore<CompleteStr, CompleteStr>,
    take_while1!(|c: char| {
        c.is_alphanumeric() || c == '_'
    })
);

named!(
    pub var<CompleteStr, String>,
    map_opt!(alnum_or_underscore, |s: CompleteStr| {
        if s.chars().next().unwrap().is_uppercase() {
            Some(s.to_string())
        } else if s.starts_with("_") && !s.starts_with("__") {
            Some(s.to_string())
        } else {
            None
        }
    })
);

fn unescape_atom(s: CompleteStr) -> Option<String> {
    let mut iter = s.chars();
    let mut buffer = String::with_capacity(s.len());

    while let Some(c) = iter.next() {
        if c == '\\' {
            if let Some(escaped) = iter.next() {
                let c = match escaped {
                    '\\' => '\\',
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\'' => '\'',
                    _ => return None // no known escape sequence, fail
                };

                buffer.push(c);
            } else {
                // There is a dangling backslash at the end, fail
                return None;
            }
        } else {
            buffer.push(c);
        }
    }

    Some(buffer)
}

// A quoted atom is of the form 'contents' where contents is an escaped string.
named!(quoted_atom<CompleteStr, String>,
    map_opt!(delimited!(tag!("'"), escaped!(none_of!("'\\"), '\\', one_of!("n'\\")), tag!("'")), unescape_atom)
);

named!(
    pub atom<CompleteStr, String>,
    alt!(quoted_atom | map_opt!(alnum_or_underscore, |s: CompleteStr| {
        if s.chars().next().unwrap().is_lowercase() {
            Some(s.to_string())
        } else {
            None
        }
    }))
);

named!(unum<CompleteStr, usize>, map_res!(digit1::<CompleteStr>, |s:CompleteStr| {
           usize::from_str_radix(&*s, 10)
        }));

named!(num<CompleteStr, i64>, map_res!(digit1::<CompleteStr>, |s:CompleteStr| {
           i64::from_str_radix(&*s, 10)
        }));

named!(
    pub pred<CompleteStr, Pred>,
    alt!(
        do_parse!(
            tag!("sys:") >>
            p: atom >>
            tag!("/") >>
            arity: unum >>
            (Pred::Sys(p, arity)))
      | map_opt!(atom, |a| {
            if &a != "sys" {
                Some(Pred::User(a))
            } else {
                None
            }
        })
    )
);

named!(
    ctor<CompleteStr, Expr<String>>,
    ws!(do_parse!(
        p: atom >>
        args: delimited!(tag!("("), separated_list_complete!(ws!(tag!(",")), expr), tag!(")")) >>
        (Expr::Ctor(p, args))
    ))
);

named!(
    pub expr<CompleteStr, Expr<String>>,
    alt!(
        map!(var, |v| { Expr::PV::<String>(v) })
      | ctor
      | map!(atom, |a| { Expr::Atom(a.to_string()) })
      | map!(num, Expr::Num)
      | ws!(delimited!(tag!("("), expr, tag!(")")))
    )
);

// A simple operator parser with hardcoded precedence
named!(conjunct<CompleteStr, Stmt<String>>,
    ws!(alt!(
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
    ))
);

named!(
    disjunct<CompleteStr, Stmt<String>>,
    map!(separated_nonempty_list!(tag!(","), conjunct), |conjuncts: Vec<Stmt<String>>| {
        conjuncts.into_iter().fold(Stmt::True, |a, b| Stmt::And(Box::new(a), Box::new(b)))
    })
);

named!(
    pub stmt<CompleteStr, Stmt<String>>,
    alt!(
        do_parse!(
            s1: disjunct >>
            ws!(tag!("->")) >>
            s2: disjunct >>
            ws!(tag!(";")) >>
            s3: disjunct >>
            (Stmt::If(Box::new(s1), Box::new(s2), Box::new(s3)))
        )
      | do_parse!(
            s1: disjunct >>
            ws!(tag!(";")) >>
            s2: disjunct >>
            (Stmt::Or(Box::new(s1), Box::new(s2)))
        )
      | disjunct
    )
);

// A top-level statement for the repl, which is a statement terminated with a "."
named!(
    pub top_level<CompleteStr, Vec<Stmt<String>>>,
    many1!(terminated!(stmt, tag!(".")))
);

named!(
    pub pred_def<CompleteStr, PredDef<String>>,
    ws!(do_parse!(
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
    ))
);

named!(
    pub program<CompleteStr, Program<String>>,
    exact!(many1!(pred_def))
);

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use Expr::*;
    use Stmt::*;

    // Fixtures

    static VALID_ATOMS: [(&str, &str); 8] = [
        ("a ", "a"),
        ("ab__C_dAA ", "ab__C_dAA"),
        ("'\\''", "'"),
        ("'\n'", "\n"),
        ("'\\n'", "\n"),
        ("';'", ";"),
        ("':-'", ":-"),
        ("' '", " "),
    ];

    static VALID_VARS: [&str; 6] = ["A", "AbCdAA", "Ab__C_dAA", "_", "_X", "X"];

    // Tests

    #[test]
    fn test_var() {

        for x in VALID_VARS.iter() {
            let mut input = x.to_string();
            input.push(' ');
            
            assert_eq!(var(CompleteStr(&input)), Ok((CompleteStr(" "), x.to_string())));
        }

        assert_eq!(var(CompleteStr("A b")), Ok((CompleteStr(" b"), "A".to_string())));

        if let Err(nom::Err::Error(_)) = var(CompleteStr("__ ")) {
        } else {
            assert!(false, "variables starting with two underscores should be rejected")
        }
    }

    #[test]
    fn test_atom() {
        for (input, atom) in VALID_ATOMS.iter() {
            let remainder = CompleteStr(if input.ends_with(" ") {
                " "
            } else {
                ""
            });
            assert_eq!(expr(CompleteStr(input)), Ok((remainder, Expr::Atom(atom.to_string()))));
        }

        assert_eq!(atom(CompleteStr("a b")), Ok((CompleteStr(" b"), "a".to_string())));

        if let Err(nom::Err::Error(_)) = atom(CompleteStr("_ ")) {
        } else {
            assert!(false, "variables starting with two underscores should be rejected")
        }
    }

    #[test]
    fn test_pred() {
        let empty = CompleteStr("");
        assert_eq!(pred(CompleteStr("foo")), Ok((empty, Pred::User("foo".to_string()))));
        assert_eq!(pred(CompleteStr("sys:foo/2")), Ok((empty, Pred::Sys("foo".to_string(), 2))));

        let invalid_arity = CompleteStr("sys:foo/-1");
        if let Err(nom::Err::Error(_)) = pred(invalid_arity) {
        } else {
            assert!(false, format!("system predicates with negative arity should be rejected, but got {:?}", pred(invalid_arity)))
        }
    }

    #[test]
    fn test_expr_atomic() {
        for (input, atom) in VALID_ATOMS.iter() {
            let remainder = CompleteStr(if input.ends_with(" ") {
                "  "
            } else {
                " "
            });

            assert_eq!(expr(CompleteStr(input)), Ok((remainder, Expr::Atom(atom.to_string()))));
        }

        for x in VALID_VARS.iter() {
            let mut input = x.to_string();
            input.push(' '); // put a delimiter whitespace at the end

            assert_eq!(expr(CompleteStr(&input)), Ok((CompleteStr(" "), Expr::PV(x.to_string()))));
        }

        // TODO: test numbers
    }

    #[test]
    fn test_expr_functor() {
        let valid_functors = vec![
            ("foo()",
             Ctor::<String>("foo".to_string(), vec![])),
            ("foo(bar)",
             Ctor::<String>("foo".to_string(), vec![Atom("bar".to_string())])),
            ("foo(Baz)",
             Ctor::<String>("foo".to_string(), vec![PV("Baz".to_string())])),
            ("foo(_)",
             Ctor::<String>("foo".to_string(), vec![PV("_".to_string())])),
            ("foo(bar,Baz)",
             Ctor::<String>("foo".to_string(), vec![Atom("bar".to_string()), PV("Baz".to_string())])),
            ("foo(bar, Baz)",
             Ctor::<String>("foo".to_string(), vec![Atom("bar".to_string()), PV("Baz".to_string())])),
            ("foo(bar, baz(quux))",
             Ctor::<String>("foo".to_string(), vec![Atom("bar".to_string()), PV("Baz".to_string())])),
        ];

        for (input, functor) in valid_functors.into_iter() {
            let remainder = CompleteStr("");
            assert_eq!(ctor(CompleteStr(input)),  Ok((remainder, functor.clone())));
            assert_eq!(expr(CompleteStr(input)),  Ok((remainder, functor)));
        }
    }
}