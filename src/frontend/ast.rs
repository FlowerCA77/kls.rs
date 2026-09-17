use std::fmt::{Display, Formatter, Result as fmtResult};

#[derive(Debug, Clone)]
pub enum ExprAST {
    Number(f64),
    Variable(String),
    Unary {
        op: String,
        operand: Box<ExprAST>,
    },
    Binary {
        op: String,
        lhs: Box<ExprAST>,
        rhs: Box<ExprAST>,
    },
    Call {
        callee: String,
        args: Vec<ExprAST>,
    },
    If {
        cond: Box<ExprAST>,
        e_true: Box<ExprAST>,
        e_false: Box<ExprAST>,
    },
    For {
        var: String,
        e_init: Box<ExprAST>,
        e_cond: Box<ExprAST>,
        e_step: Box<ExprAST>,
        e_body: Box<ExprAST>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FunctionName {
    Ident(String),
    Unary(char, u8),
    Binary(char, u8),
}

impl FunctionName {
    pub fn llvm_name(&self) -> String {
        match self {
            FunctionName::Ident(s) => s.clone(),
            FunctionName::Unary(op, _) => format!("unary_{:02x}", *op as u32),
            FunctionName::Binary(op, _) => format!("binary_{:02x}", *op as u32),
        }
    }
}

impl Display for FunctionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmtResult {
        match self {
            FunctionName::Ident(s) => write!(f, "{s}"),
            FunctionName::Unary(op, prec) => write!(f, "unary({op}, {prec})"),
            FunctionName::Binary(op, prec) => write!(f, "binary({op}, {prec})"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrototypeAST {
    pub(crate) name: FunctionName,
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
