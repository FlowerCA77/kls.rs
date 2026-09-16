use crate::Result;
use crate::codegen::Codegen;
use crate::frontend::ast::{FunctionAST, PrototypeAST};

use inkwell::values::FunctionValue;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_prototype(
        &mut self,
        proto: &PrototypeAST,
    ) -> Result<FunctionValue<'ctx>> {
        let function = if let Some(f) = self.module.get_function(&proto.name) {
            f
        } else {
            let f64_type = self.context.f64_type();
            let param_types: Vec<_> = proto.args.iter().map(|_| f64_type.into()).collect();
            let fn_type = f64_type.fn_type(&param_types, false);
            self.module.add_function(&proto.name, fn_type, None)
        };

        self.function_protos
            .insert(proto.name.clone(), proto.clone());

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

        let body = self.compile_expr(&func.body)?;
        self.builder.build_return(Some(&body))?;

        if function.verify(true) {
            Ok(function)
        } else {
            unsafe {
                function.delete();
            }
            Err(format!("invalid function: {}", func.proto.name).into())
        }
    }
}
