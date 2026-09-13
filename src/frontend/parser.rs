use crate::frontend::ast::{ExprAST, FunctionAST, KEYWORDS, Program, PrototypeAST, TopLevel};
use chumsky::{
    error::Rich,
    pratt::{infix, left},
    prelude::*,
};

fn create_identifier_parser<'src>()
-> impl Parser<'src, &'src str, String, extra::Err<Rich<'src, char>>> + Clone {
    text::ident()
        .filter(|s: &&str| !KEYWORDS.contains(s))
        .map(|s: &str| s.to_string())
}

fn create_prototype_parser<'src>()
-> impl Parser<'src, &'src str, PrototypeAST, extra::Err<Rich<'src, char>>> + Clone {
    create_identifier_parser()
        .padded()
        .then(
            create_identifier_parser()
                .padded()
                .separated_by(just(','))
                .collect::<Vec<_>>()
                .delimited_by(just('('), just(')')),
        )
        .padded()
        .map(|(name, args)| PrototypeAST { name, args })
}

fn create_def_parser<'src>()
-> impl Parser<'src, &'src str, (PrototypeAST, ExprAST), extra::Err<Rich<'src, char>>> + Clone {
    text::keyword("def")
        .padded()
        .ignore_then(create_prototype_parser())
        .then(create_expr_parser())
        .padded()
}

fn create_ext_parser<'src>()
-> impl Parser<'src, &'src str, PrototypeAST, extra::Err<Rich<'src, char>>> + Clone {
    text::keyword("extern")
        .padded()
        .ignore_then(create_prototype_parser())
        .padded()
}

fn create_expr_parser<'src>()
-> impl Parser<'src, &'src str, ExprAST, extra::Err<Rich<'src, char>>> + Clone {
    recursive(|expr| {
        let number_parser = text::int(10)
            .then(just('.').ignore_then(text::digits(10)).or_not())
            .to_slice()
            .map(|s: &'src str| ExprAST::Number(s.parse().unwrap()));

        let identifier_parser = create_identifier_parser();

        let call_parser = identifier_parser
            .clone()
            .then(
                expr.clone()
                    .separated_by(just(','))
                    .collect::<Vec<ExprAST>>()
                    .delimited_by(just('('), just(')')),
            )
            .map(|(callee, args)| ExprAST::Call { callee, args });

        let variable_parser = identifier_parser.map(ExprAST::Variable);

        let atom_parser = choice((
            number_parser,
            call_parser,
            variable_parser,
            expr.clone().delimited_by(just('('), just(')')),
        ))
        .padded();

        atom_parser.pratt((
            infix(left(10), just('<'), |lhs, _, rhs, _| ExprAST::Binary {
                op: '<',
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(20), just('+'), |lhs, _, rhs, _| ExprAST::Binary {
                op: '+',
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(20), just('-'), |lhs, _, rhs, _| ExprAST::Binary {
                op: '-',
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(40), just('*'), |lhs, _, rhs, _| ExprAST::Binary {
                op: '*',
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(40), just('/'), |lhs, _, rhs, _| ExprAST::Binary {
                op: '/',
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
        ))
    })
    .padded()
}

pub fn create_top_level_parser<'src>()
-> impl Parser<'src, &'src str, TopLevel, extra::Err<Rich<'src, char>>> + Clone {
    let def_parser =
        create_def_parser().map(|(proto, body)| TopLevel::Def(FunctionAST { proto, body }));

    let ext_parser = create_ext_parser().map(TopLevel::Extern);

    let expr_parser = create_expr_parser().map(TopLevel::Expr);

    choice((def_parser, ext_parser, expr_parser)).padded()
}

pub fn create_program_parser<'src>()
-> impl Parser<'src, &'src str, Program, extra::Err<Rich<'src, char>>> + Clone {
    create_top_level_parser()
        .repeated()
        .collect::<Vec<TopLevel>>()
        .map(|items| Program { items })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_level_parser() {
        let result = create_top_level_parser()
            .parse("def foo(x, y) x + y")
            .into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    #[test]
    fn test_def_parser() {
        let result = create_def_parser()
            .parse("def foo(x, y) x + y")
            .into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    #[test]
    fn test_padded_def() {
        let result = create_top_level_parser()
            .parse(" def    foo(  x  ,  y  )    x   +   y  ")
            .into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    #[test]
    fn test_padded_extern() {
        let result = create_top_level_parser()
            .parse(" extern   sin ( a )  ")
            .into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    #[test]
    fn test_padded_expr() {
        let result = create_top_level_parser()
            .parse(" 1 +  (  2    * 3   )  ")
            .into_result();
        assert!(result.is_ok(), "parse failed: {:#?}", result);
    }

    fn fast_parses(input: &str) -> bool {
        create_top_level_parser().parse(input).into_result().is_ok()
    }

    #[test]
    fn test_valid_inputs() {
        assert!(fast_parses("def foo(x, y) x + y"));
        assert!(fast_parses("extern sin(a)"));
        assert!(fast_parses("1 + 2 * 3"));
        assert!(fast_parses("foo(1, 2, 3)"));
        assert!(fast_parses("(1 + 2) * 3"));
    }

    #[test]
    fn test_invalid_inputs() {
        assert!(!fast_parses("def foo(x"));
        assert!(!fast_parses("1 +"));
        assert!(!fast_parses(")"));
        assert!(!fast_parses("def"));
    }

    #[test]
    fn test_whitespace_variants() {
        assert!(fast_parses("def    foo(  x  ,  y  )    x   +   y"));
        assert!(fast_parses("1+2"));
        assert!(fast_parses("  1  +  2  "));
    }

    /// lightweight fuzz
    #[test]
    fn test_no_panic_on_garbage() {
        let garbage = [
            "",
            " ",
            "\n",
            "?",
            "@#$",
            "def def",
            "((((",
            "))))",
            "1 + + 2",
            "foo()",
            "def foo()",
            "extern",
            "\0\0\0",
            "💥",
            "def foo(x) x + ",
            "4.",
            "4.5.6",
            ".5",
            "99999999999999999999999999999999999",
        ];

        for input in garbage {
            let _ = create_top_level_parser().parse(input).into_result();
        }
    }

    #[test]
    fn test_keywords_not_identifiers() {
        assert!(!fast_parses("def"));
        assert!(!fast_parses("extern"));
        assert!(!fast_parses("def(x) x"));
        assert!(!fast_parses("def def(x) x"));
        assert!(fast_parses("def foo(x) x"));
        assert!(!fast_parses("def foo(def) def"));
    }
}
