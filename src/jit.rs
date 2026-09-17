use crate::Result;

use inkwell::{
    OptimizationLevel,
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
    targets::TargetMachine,
};

pub(crate) struct Jit<'ctx> {
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> Jit<'ctx> {
    pub fn new(
        context: &'ctx Context,
        target_machine: &TargetMachine,
        opt_level: OptimizationLevel,
    ) -> Result<Self> {
        let init_module = context.create_module("kaleidoscope_jit_init");
        init_module.set_triple(&target_machine.get_triple());
        init_module.set_data_layout(&target_machine.get_target_data().get_data_layout());
        let execution_engine = init_module.create_jit_execution_engine(opt_level)?;
        Ok(Self { execution_engine })
    }

    pub fn add_module(&mut self, module: Module<'ctx>) -> Result<()> {
        self.execution_engine
            .add_module(&module)
            .map_err(|_| {
                format!(
                    "cannot add module {} to jit",
                    module.get_name().to_str().unwrap_or("<unknown module>")
                )
            })
            .map_err(Into::into)
    }

    pub fn lookup(&self, name: &str) -> Option<JitFunction<'ctx, unsafe extern "C" fn() -> f64>> {
        unsafe {
            match self.execution_engine.get_function(name) {
                Ok(f) => Some(f),
                Err(e) => {
                    eprintln!("lookup `{}` failed: {:?}", name, e);
                    None
                }
            }
        }
    }
}
