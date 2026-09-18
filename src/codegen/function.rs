use crate::{
    Result,
    codegen::Codegen,
    frontend::ast::{FunctionAST, PrototypeAST},
};
use inkwell::values::FunctionValue;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_prototype(
        &mut self,
        proto: &PrototypeAST,
    ) -> Result<FunctionValue<'ctx>> {
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

        self.named_values.clear();

        for (i, arg) in function.get_param_iter().enumerate() {
            self.named_values.insert(func.proto.args[i].clone(), arg);
        }

        match self.compile_expr(&func.body) {
            Ok(body) => {
                if let Err(e) = self.builder.build_return(Some(&body)) {
                    unsafe { function.delete() };
                    return Err(e.into());
                }

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
                unsafe { function.delete() };
                return Err(e);
            }
        }
    }

    pub fn compile_extern(&mut self, proto: &PrototypeAST) -> Result<FunctionValue<'ctx>> {
        let function = self.compile_prototype(proto)?;

        self.function_protos
            .insert(proto.name.llvm_name(), proto.clone());

        Ok(function)
    }
}
