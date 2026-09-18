pub mod expr;
pub mod function;
pub mod optimize;
pub mod top_level;

use crate::frontend::ast::PrototypeAST;
use inkwell::{
    builder::Builder, context::Context, module::Module, targets::TargetMachine,
    values::BasicValueEnum,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default)]
pub struct CodegenOptions {
    pub fast_math: bool,
}

pub(crate) struct Codegen<'ctx> {
    pub(crate) anon_counter: usize,
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) target_machine: TargetMachine,
    pub(crate) named_values: HashMap<String, BasicValueEnum<'ctx>>,
    pub(crate) function_protos: HashMap<String, PrototypeAST>,
    pub(crate) options: CodegenOptions,
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn new(
        context: &'ctx Context,
        module_name: &str,
        target_machine: TargetMachine,
        options: CodegenOptions,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let named_values = HashMap::new();
        let function_protos = HashMap::new();

        module.set_triple(&target_machine.get_triple());
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        Self {
            anon_counter: 0,
            context,
            module,
            builder,
            target_machine,
            named_values,
            function_protos,
            options,
        }
    }

    pub(crate) fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub(crate) fn take_module(&mut self) -> Module<'ctx> {
        let module_name = self
            .module
            .get_name()
            .to_str()
            .unwrap_or("kaleidoscope_module")
            .to_string();
        let new_module = self.context.create_module(module_name.as_str());
        new_module.set_triple(&self.target_machine.get_triple());
        new_module.set_data_layout(&self.target_machine.get_target_data().get_data_layout());
        std::mem::replace(&mut self.module, new_module)
    }

    pub(crate) fn get_target_machine(&self) -> &TargetMachine {
        &self.target_machine
    }
}
