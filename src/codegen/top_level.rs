use crate::Result;
use crate::codegen::Codegen;
use crate::frontend::ast::{FunctionAST, FunctionName, PrototypeAST, TopLevel};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_top_level(&mut self, item: &TopLevel) -> Result<()> {
        match item {
            TopLevel::Def(f) => {
                self.compile_function(f)?;
                Ok(())
            }
            TopLevel::Extern(p) => {
                self.compile_extern(p)?;
                Ok(())
            }
            TopLevel::Expr(e) => {
                let func = FunctionAST {
                    proto: PrototypeAST {
                        name: FunctionName::Ident("__anon_expr".to_string()),
                        args: vec![],
                    },
                    body: e.clone(),
                };
                self.compile_function(&func)?;
                Ok(())
            }
        }
    }
}
