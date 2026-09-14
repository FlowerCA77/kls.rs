use crate::frontend::ast::PrototypeAST;
use inkwell::{builder::Builder, context::Context, module::Module, values::BasicValueEnum};
use std::collections::HashMap;

pub(crate) struct Codegen<'ctx> {
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) named_values: HashMap<String, BasicValueEnum<'ctx>>,
    pub(crate) function_protos: HashMap<String, PrototypeAST>,
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self {
            context,
            module: context.create_module(module_name),
            builder: context.create_builder(),
            named_values: HashMap::new(),
            function_protos: HashMap::new(),
        }
    }

    pub(crate) fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }
}
