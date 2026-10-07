use std::path::PathBuf;

use chumsky::prelude::*;

use crate::Result;
use crate::codegen::Codegen;
use crate::frontend::parser::create_program_parser;

pub fn compile_file(codegen: &mut Codegen, path: &PathBuf) -> Result<()> {
    let src = std::fs::read_to_string(path)?;
    let program = create_program_parser()
        .parse(src.as_str())
        .into_result()
        .map_err(|errs| {
            errs.iter()
                .map(|e| format!("{}: {}", path.display(), e))
                .collect::<Vec<_>>()
                .join("\n")
        })?;

    for item in &program.items {
        codegen.compile_top_level(item)?;
    }
    Ok(())
}

pub fn emit_file(codegen: &Codegen, file_type: inkwell::targets::FileType, path: &PathBuf) -> Result<()> {
    codegen
        .get_target_machine()
        .write_to_file(codegen.get_module(), file_type, path)?;
    Ok(())
}
