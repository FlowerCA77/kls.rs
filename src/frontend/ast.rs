//! Kaleidoscope SPEC:
//! ```plaintext
//! <Program>              ::= <Top-Level Expression>*
//!
//! <Top-Level Expression> ::= <Function> | <Prototype> | <Expression>
//!
//! <Function>             ::= def <Prototype> <Expression>
//! <Prototype>            ::= <id> ( <id>* )
//! <Expression>           ::= <Number> | <Variable> | <Binary> | <Call>
//!
//! <Number>               ::=
//! <Variable>             ::=
//! <Binary>               ::=
//! <Call>                 ::=
//! ```

pub(crate) const KEYWORDS: &[&str] = &["def", "extern"];

#[derive(Debug, Clone)]
pub enum ExprAST {
    Number(f64),
    Variable(String),
    Binary {
        op: char,
        lhs: Box<ExprAST>,
        rhs: Box<ExprAST>,
    },
    Call {
        callee: String,
        args: Vec<ExprAST>,
    },
}

#[derive(Debug, Clone)]
pub struct PrototypeAST {
    pub(crate) name: String,
    pub(crate) args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionAST {
    pub(crate) proto: PrototypeAST,
    pub(crate) body: ExprAST,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Def(FunctionAST),
    Extern(PrototypeAST),
    Expr(ExprAST),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub(crate) items: Vec<TopLevel>,
}
