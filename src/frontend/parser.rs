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

pub(crate) fn create_top_level_parser<'src>()
-> impl Parser<'src, &'src str, TopLevel, extra::Err<Rich<'src, char>>> + Clone {
    let def_parser =
        create_def_parser().map(|(proto, body)| TopLevel::Def(FunctionAST { proto, body }));

    let ext_parser = create_ext_parser().map(TopLevel::Extern);

    let expr_parser = create_expr_parser().map(TopLevel::Expr);

    choice((def_parser, ext_parser, expr_parser)).padded()
}

pub(crate) fn create_program_parser<'src>()
-> impl Parser<'src, &'src str, Program, extra::Err<Rich<'src, char>>> + Clone {
    create_top_level_parser()
        .repeated()
        .collect::<Vec<TopLevel>>()
        .map(|items| Program { items })
}

#[path = "./tests/test_parser.rs"]
#[cfg(test)]
mod parser_tests;
