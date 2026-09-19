use inkwell::passes::PassBuilderOptions;

use crate::Result;
use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn optimize(&self, pipeline: &str) -> Result<()> {
        self.module
            .run_passes(pipeline, &self.target_machine, PassBuilderOptions::create())
            .map_err(Into::into)
    }
}
