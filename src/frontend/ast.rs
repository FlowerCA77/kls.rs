pub(crate) const KEYWORDS: &[&str] = &["def", "extern", "if", "then", "else", "for", "in"];

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
