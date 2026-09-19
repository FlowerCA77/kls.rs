use std::collections::HashMap;

use inkwell::values::FunctionValue;

use crate::Result;
use crate::codegen::{BindingValue, Codegen};
use crate::frontend::ast::{FunctionAST, PrototypeAST};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_prototype(&mut self, proto: &PrototypeAST) -> Result<FunctionValue<'ctx>> {
        let llvm_name = proto.name.llvm_name();

        let function = if let Some(f) = self.module.get_function(&llvm_name) {
            f
        } else {
            let f64_type = self.context.f64_type();
            let param_types: Vec<_> = proto.args.iter().map(|_| f64_type.into()).collect();
            let fn_type = f64_type.fn_type(&param_types, false);

            self.module.add_function(&llvm_name, fn_type, None)
        };

        for (i, arg) in function.get_param_iter().enumerate() {
            arg.set_name(&proto.args[i]);
        }

        Ok(function)
    }

    pub(crate) fn compile_function(&mut self, func: &FunctionAST) -> Result<FunctionValue<'ctx>> {
        let function = self.compile_prototype(&func.proto)?;

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        self.scopes.push(HashMap::new());

        for (i, arg) in function.get_param_iter().enumerate() {
            let alloca = self.create_entry_block_alloca(&func.proto.args[i])?;
            self.builder.build_store(alloca, arg)?;
            self.scopes
                .last_mut()
                .ok_or("compile error")?
                .insert(func.proto.args[i].clone(), BindingValue::Alloca(alloca));
        }

        match self.compile_expr(&func.body) {
            Ok(body) => {
                if let Err(e) = self.builder.build_return(Some(&body)) {
                    self.scopes.pop();
                    unsafe { function.delete() };
                    return Err(e.into());
                }

                self.scopes.pop();

                if function.verify(true) {
                    self.function_protos
                        .insert(func.proto.name.llvm_name(), func.proto.clone());
                    Ok(function)
                } else {
                    unsafe { function.delete() };
                    return Err(format!("invalid function: {}", func.proto.name.llvm_name()).into());
                }
            }
            Err(e) => {
                self.scopes.pop();
                unsafe { function.delete() };
                return Err(e);
            }
        }
    }

    pub fn compile_extern(&mut self, proto: &PrototypeAST) -> Result<FunctionValue<'ctx>> {
        let function = self.compile_prototype(proto)?;

        self.function_protos.insert(proto.name.llvm_name(), proto.clone());

        Ok(function)
    }
}
