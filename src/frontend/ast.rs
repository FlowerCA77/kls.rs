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
    Block(Vec<ExprAST>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FunctionName {
    Ident(String),
    Unary(String),
    Binary(String),
}

impl FunctionName {
    pub fn llvm_name(&self) -> String {
        match self {
            FunctionName::Ident(s) => s.clone(),
            FunctionName::Unary(op) => format!(
                "unary_{}",
                op.chars().map(|c| format!("{:02x}", c as u32)).collect::<String>()
            ),
            FunctionName::Binary(op) => format!(
                "binary_{}",
                op.chars().map(|c| format!("{:02x}", c as u32)).collect::<String>()
            ),
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
