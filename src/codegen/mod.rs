pub mod expr;
pub mod function;
pub mod optimize;
pub mod top_level;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::TargetMachine;
use inkwell::values::{BasicValueEnum, PointerValue};

use crate::Result;
use crate::frontend::ast::PrototypeAST;

pub(crate) const STDLIB_SRC: &str = include_str!("../stdlib.kls");

#[derive(Debug, Clone, Copy, Default)]
pub struct CodegenOptions {
    pub fast_math: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum BindingValue<'ctx> {
    Alloca(PointerValue<'ctx>),
    Value(BasicValueEnum<'ctx>),
    Global,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ImportMarker {
    Path(PathBuf),
    Named(String),
}

pub(crate) struct Codegen<'ctx> {
    pub(crate) anon_counter: usize,
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) imported: HashSet<ImportMarker>,
    pub(crate) target_machine: TargetMachine,
    pub(crate) scopes: Vec<HashMap<String, BindingValue<'ctx>>>,
    pub(crate) globals: HashSet<String>,
    pub(crate) pending_inits: Vec<String>,
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
        let scopes = Vec::new();
        let globals = HashSet::new();
        let pending_inits = Vec::new();
        let function_protos = HashMap::new();
        let imported = HashSet::new();

        module.set_triple(&target_machine.get_triple());
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        Self {
            anon_counter: 0,
            context,
            module,
            builder,
            imported,
            target_machine,
            scopes,
            globals,
            pending_inits,
            function_protos,
            options,
        }
    }

    pub(crate) fn get_module(&self) -> &Module<'ctx> { &self.module }

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

        for name in &self.globals {
            let g = new_module.add_global(self.context.f64_type(), None, name);
            g.set_linkage(Linkage::External);
        }

        std::mem::replace(&mut self.module, new_module)
    }

    pub(crate) fn get_target_machine(&self) -> &TargetMachine { &self.target_machine }

    pub(crate) fn lookup_variable(&self, name: &str) -> Option<BindingValue<'ctx>> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.get(name) {
                return Some(*b);
            }
        }

        Some(BindingValue::Global)
    }

    pub(crate) fn create_entry_block_alloca(&self, name: &str) -> Result<PointerValue<'ctx>> {
        let function = self
            .builder
            .get_insert_block()
            .ok_or("llvm error")?
            .get_parent()
            .ok_or("llvm error")?;

        let entry = function.get_first_basic_block().ok_or("llvm error")?;

        let tmp_builder = self.context.create_builder();

        match entry.get_first_instruction() {
            Some(first) => tmp_builder.position_before(&first),
            None => tmp_builder.position_at_end(entry),
        }

        tmp_builder
            .build_alloca(self.context.f64_type(), name)
            .map_err(Into::into)
    }
}
