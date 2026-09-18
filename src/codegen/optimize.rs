use crate::{Result, codegen::Codegen};
use inkwell::passes::PassBuilderOptions;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn optimize(&self, pipeline: &str) -> Result<()> {
        self.module
            .run_passes(pipeline, &self.target_machine, PassBuilderOptions::create())
            .map_err(Into::into)
    }
}
