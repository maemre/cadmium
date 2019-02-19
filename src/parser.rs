// Parser for the high-level AST

use ::nom::*;
use crate::ast_common::*;
use crate::ast::*;

named!(
    alnum_or_underscore<&str, &str>,
    take_while1!(|c: char| {
        c.is_alphanumeric() || c == '_'
    })
);

named!(
    pub var<&str, String>,
    map_opt!(alnum_or_underscore, |s: &str| {
        if s.chars().next().unwrap().is_uppercase() {
            Some(s.to_string())
        } else if s.starts_with("_") && !s.starts_with("__") {
            Some(s.to_string())
        } else {
            None
        }
    })
);

fn unescape_atom(s: &str) -> Option<String> {
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
named!(quoted_atom<&str, String>,
    map_opt!(delimited!(tag!("'"), escaped!(none_of!("'\\"), '\\', one_of!("n'\\")), tag!("'")), unescape_atom)
);

named!(
    pub atom<&str, String>,
    alt!(quoted_atom | map_opt!(alnum_or_underscore, |s: &str| {
        if s.chars().next().unwrap().is_lowercase() {
            Some(s.to_string())
        } else {
            None
        }
    }))
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
      | map_opt!(atom, |a| {
            if &a != "sys" {
                Some(Pred::User(a))
            } else {
                None
            }
        })
    )
);

// TODO: fix whitespace issues
named!(
    ctor<&str, Expr<String>>,
    do_parse!(
        p: atom >>
        args: delimited!(tag!("("), separated_list_complete!(tag!(","), expr), tag!(")")) >>
        (Expr::Ctor(p, args))
    )
);

named!(
    pub expr<&str, Expr<String>>,
    alt!(
        map!(var, |v| { Expr::PV::<String>(v) })
      | ctor
      // when choosing an atom, make sure that it's not followed by an '('
      | terminated!(map!(atom, |a| { Expr::Atom(a.to_string()) } ), peek!(none_of!("(")))
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

    // Fixtures

    static VALID_ATOMS: [(&str, &str); 7] = [
        ("a ", "a"),
        ("ab__C_dAA ", "ab__C_dAA"),
        ("'\\''", "'"),
        ("'\n'", "\n"),
        ("'\\n'", "\n"),
        ("';'", ";"),
        ("':-'", ":-"),
    ];

    static VALID_VARS: [&str; 6] = ["A", "AbCdAA", "Ab__C_dAA", "_", "_X", "X"];

    // Tests

    #[test]
    fn test_var() {

        for x in VALID_VARS.iter() {
            let mut input = x.to_string();
            input.push(' ');
            
            assert_eq!(var(&input), Ok((" ", x.to_string())));
        }

        assert_eq!(var("A b"), Ok((" b", "A".to_string())));

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
        assert_eq!(atom("'\\''"), Ok(("", "'".to_string())));
        assert_eq!(atom("':-'"), Ok(("", ":-".to_string())));
        assert_eq!(atom("'\n'"), Ok(("", "\n".to_string())));
        assert_eq!(atom("'\\n'"), Ok(("", "\n".to_string())));
        assert_eq!(atom("';'"), Ok(("", ";".to_string())));

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
            assert!(false, format!("system predicates with negative arity should be rejected, but got {:?}", pred("sys:foo/-1")))
        }
    }

    #[test]
    fn test_expr_atomic() {
        for (input, atom) in VALID_ATOMS.iter() {
            let remainder = if input.ends_with(" ") {
                "  "
            } else {
                " "
            };

            let mut input_owned = input.to_string();
            input_owned.push(' '); // put a delimiter whitespace at the end to finish parsing

            assert_eq!(expr(&input_owned), Ok((remainder, Expr::Atom(atom.to_string()))));
        }

        for x in VALID_VARS.iter() {
            let mut input = x.to_string();
            input.push(' '); // put a delimiter whitespace at the end

            assert_eq!(expr(&input), Ok((" ", Expr::PV(x.to_string()))));
        }

        // TODO: test numbers
    }

    #[test]
    fn test_expr_functor() {
        use Expr::*;

        let valid_functors = vec![
            ("foo()".to_string(),
             Ctor::<String>("foo".to_string(), vec![])),
            ("foo(bar)".to_string(),
             Ctor::<String>("foo".to_string(), vec![Atom("bar".to_string())])),
            ("foo(Baz)".to_string(),
             Ctor::<String>("foo".to_string(), vec![PV("Baz".to_string())])),
            ("foo(_)".to_string(),
             Ctor::<String>("foo".to_string(), vec![PV("_".to_string())])),
            ("foo(bar,Baz)".to_string(),
             Ctor::<String>("foo".to_string(), vec![Atom("bar".to_string()), PV("Baz".to_string())])),
            ("foo(bar, Baz)".to_string(),
             Ctor::<String>("foo".to_string(), vec![Atom("bar".to_string()), PV("Baz".to_string())]))
        ];

        for (mut input, functor) in valid_functors.into_iter() {
            input.push(' ');
            assert_eq!(ctor(&input),  Ok((" ", functor.clone())));
            assert_eq!(expr(&input),  Ok((" ", functor)));
        }
    }
}