use chumsky::error::Rich;
use chumsky::pratt::{infix, left, prefix};
use chumsky::prelude::*;

use crate::frontend::ast::{ExprAST, FunctionAST, FunctionName, Program, PrototypeAST, TopLevel};

pub(crate) const KEYWORDS: &[&str] = &["def", "extern", "if", "then", "else", "for", "in"];
pub(crate) const BUILTIN_UNARY_OPS: &[&str] = &["-", "+"];
pub(crate) const BUILTIN_BINARY_OPS: &[&str] = &["+", "-", "*", "/", "<", "<=", ">", ">="];

fn is_op_char(c: char) -> bool { "+-*/<>=&|^~!%@$?_".contains(c) }

fn create_identifier_parser<'src>() -> impl Parser<'src, &'src str, String, extra::Err<Rich<'src, char>>> + Clone {
    text::ident()
        .filter(|s: &&str| !KEYWORDS.contains(s))
        .map(|s: &str| s.to_string())
}

fn create_prototype_parser<'src>() -> impl Parser<'src, &'src str, PrototypeAST, extra::Err<Rich<'src, char>>> + Clone {
    let op_name = just('(')
        .ignore_then(
            any::<&str, extra::Err<Rich<'src, char>>>()
                .filter(|c: &char| is_op_char(*c))
                .repeated()
                .at_least(1)
                .to_slice(),
        )
        .then_ignore(just(')'))
        .map(|op: &str| op.to_string());

    let args = create_identifier_parser()
        .padded()
        .separated_by(just(','))
        .collect::<Vec<String>>()
        .delimited_by(just('('), just(')'));

    let func_def = text::ident()
        .filter(|s: &&str| !KEYWORDS.contains(s))
        .map(|s: &str| FunctionName::Ident(s.to_string()))
        .then(args.clone());

    let op_def = op_name.then(args).try_map(|(op, args), span| match args.len() {
        1 => {
            if BUILTIN_UNARY_OPS.contains(&op.as_str()) {
                Err(Rich::custom(span, format!("{} is a builtin operator", op)))
            } else {
                Ok((FunctionName::Unary(op), args))
            }
        }
        2 => {
            if BUILTIN_BINARY_OPS.contains(&op.as_str()) {
                Err(Rich::custom(span, format!("{} is a builtin operator", op)))
            } else {
                Ok((FunctionName::Binary(op), args))
            }
        }
        n => Err(Rich::custom(span, format!("operator must have 1 or 2 args, got {}", n))),
    });

    func_def
        .or(op_def)
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

fn create_ext_parser<'src>() -> impl Parser<'src, &'src str, PrototypeAST, extra::Err<Rich<'src, char>>> + Clone {
    text::keyword("extern")
        .padded()
        .ignore_then(create_prototype_parser())
        .padded()
}

fn create_expr_parser<'src>() -> impl Parser<'src, &'src str, ExprAST, extra::Err<Rich<'src, char>>> + Clone {
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

        let if_parser = text::keyword("if")
            .padded()
            .ignore_then(expr.clone())
            .then_ignore(text::keyword("then").padded())
            .then(expr.clone())
            .then_ignore(text::keyword("else").padded())
            .then(expr.clone())
            .map(|((cond, e_true), e_false)| ExprAST::If {
                cond: Box::new(cond),
                e_true: Box::new(e_true),
                e_false: Box::new(e_false),
            });

        let for_parser = text::keyword("for")
            .padded()
            .ignore_then(text::ident().padded().map(|s: &str| s.to_string()))
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(just(',').padded())
            .then(expr.clone())
            .then_ignore(just(',').padded())
            .then(expr.clone())
            .then_ignore(text::keyword("in").padded())
            .then(expr.clone())
            .map(|((((var, e_init), e_cond), e_step), e_body)| ExprAST::For {
                var,
                e_init: Box::new(e_init),
                e_cond: Box::new(e_cond),
                e_step: Box::new(e_step),
                e_body: Box::new(e_body),
            });

        let block_parser = expr
            .clone()
            .separated_by(just(";").or_not())
            .collect::<Vec<ExprAST>>()
            .delimited_by(just('{'), just('}'))
            .map(ExprAST::Block);

        let atom_parser = choice((
            number_parser,
            if_parser,
            for_parser,
            call_parser,
            variable_parser,
            block_parser,
            expr.clone().delimited_by(just('('), just(')')),
        ))
        .padded();

        atom_parser.pratt((
            prefix(50, just('-'), |_, rhs, _| ExprAST::Unary {
                op: "-".to_string(),
                operand: Box::new(rhs),
            }),
            prefix(50, just('+'), |_, rhs, _| ExprAST::Unary {
                op: "+".to_string(),
                operand: Box::new(rhs),
            }),
            infix(left(10), just("<="), |lhs, _, rhs, _| ExprAST::Binary {
                op: "<=".to_string(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(10), just(">="), |lhs, _, rhs, _| ExprAST::Binary {
                op: ">=".to_string(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(10), just('<'), |lhs, _, rhs, _| ExprAST::Binary {
                op: "<".to_string(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(10), just('>'), |lhs, _, rhs, _| ExprAST::Binary {
                op: ">".to_string(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(20), just('+'), |lhs, _, rhs, _| ExprAST::Binary {
                op: "+".to_string(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(20), just('-'), |lhs, _, rhs, _| ExprAST::Binary {
                op: "-".to_string(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(40), just('*'), |lhs, _, rhs, _| ExprAST::Binary {
                op: "*".to_string(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            infix(left(40), just('/'), |lhs, _, rhs, _| ExprAST::Binary {
                op: "/".to_string(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            prefix(
                50,
                any::<&str, extra::Err<Rich<'src, char>>>()
                    .filter(|c: &char| is_op_char(*c))
                    .repeated()
                    .at_least(1)
                    .to_slice(),
                |op: &str, rhs, _| ExprAST::Unary {
                    op: op.to_string(),
                    operand: Box::new(rhs),
                },
            ),
            infix(
                left(30),
                any::<&str, extra::Err<Rich<'src, char>>>()
                    .filter(|c: &char| is_op_char(*c))
                    .repeated()
                    .at_least(1)
                    .to_slice(),
                |lhs, op: &str, rhs, _| ExprAST::Binary {
                    op: op.to_string(),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            ),
        ))
    })
    .padded()
}

pub(crate) fn create_top_level_parser<'src>()
-> impl Parser<'src, &'src str, TopLevel, extra::Err<Rich<'src, char>>> + Clone {
    let def_parser = create_def_parser().map(|(proto, body)| TopLevel::Def(FunctionAST { proto, body }));

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
