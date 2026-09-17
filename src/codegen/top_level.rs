use crate::Result;
use crate::codegen::Codegen;
use crate::frontend::ast::{FunctionAST, PrototypeAST, TopLevel};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_top_level(&mut self, item: &TopLevel) -> Result<Option<String>> {
        match item {
            TopLevel::Def(f) => {
                let f = self.compile_function(f)?;
                match f.get_name().to_str() {
                    Ok(fn_name) => Ok(Some(fn_name.to_string())),
                    Err(_) => Ok(None),
                }
            }
            TopLevel::Extern(p) => {
                let f = self.compile_extern(p)?;
                match f.get_name().to_str() {
                    Ok(fn_name) => Ok(Some(fn_name.to_string())),
                    Err(_) => Ok(None),
                }
            }
            TopLevel::Expr(e) => {
                let name = format!("__anon_expr_{}", self.anon_counter);
                let func = FunctionAST {
                    proto: PrototypeAST {
                        name: name.clone(),
                        args: vec![],
                    },
                    body: e.clone(),
                };
                self.anon_counter += 1;
                self.compile_function(&func)?;
                Ok(Some(name))
            }
        }
    }
}
