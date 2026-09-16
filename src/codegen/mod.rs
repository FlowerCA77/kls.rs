pub mod expr;
pub mod function;
pub mod optimize;
pub mod top_level;

use crate::frontend::ast::PrototypeAST;

use std::collections::HashMap;

use inkwell::{
    builder::Builder, context::Context, module::Module, targets::TargetMachine,
    values::BasicValueEnum,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct CodegenOptions {
    pub fast_math: bool,
}

pub(crate) struct Codegen<'ctx> {
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
        Self {
            context,
            module: context.create_module(module_name),
            builder: context.create_builder(),
            target_machine,
            named_values: HashMap::new(),
            function_protos: HashMap::new(),
            options,
        }
    }

    pub(crate) fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub(crate) fn get_target_machine(&self) -> &TargetMachine {
        &self.target_machine
    }
}
