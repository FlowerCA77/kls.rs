mod backend;
mod cli;
mod codegen;
mod frontend;
pub mod stdlib;

use std::path::PathBuf;

use clap::Parser;
use inkwell::context::Context;

use crate::backend::compiler::{compile_file, emit_file};
use crate::backend::jit::Jit;
use crate::backend::linker::link;
use crate::backend::repl::{ReplContext, run_repl};
use crate::backend::target::create_target_machine;
use crate::cli::{Cli, EmitKind};
use crate::codegen::{Codegen, CodegenOptions};
use crate::frontend::ast::TopLevel;

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let opt_level = cli.optimization_level();
    eprintln!("Optimization level: {} => {:#?}", cli.optimize, opt_level);

    if cli.dry_run {
        eprintln!("WARNING: dry run");
    }

    let context = Context::create();

    let target_machine = create_target_machine(opt_level);

    let options = CodegenOptions {
        fast_math: cli.fast_math,
    };

    if cli.files.is_empty() {
        let mut codegen = Codegen::new(&context, "kaleidoscope_repl", target_machine, options);

        let mut jit = Jit::new(&context, codegen.get_target_machine(), opt_level).unwrap();

        codegen.compile_top_level(&TopLevel::Import("std".into()))?;

        if cli.ir {
            println!("=== Stdlib IR === (stderr)");
            codegen.get_module().print_to_stderr();
        }

        jit.add_module(codegen.take_module())?;

        println!("=== Repl === (stdout)");
        run_repl(&mut ReplContext::new(codegen, jit), &cli, opt_level)?;
    } else {
        let mut codegen = Codegen::new(&context, "kaleidoscope", target_machine, options);

        for path in &cli.files {
            compile_file(&mut codegen, path)?;
        }

        let pipeline = format!("default<O{}>", cli.optimize);
        codegen.optimize(&pipeline)?;

        let output = cli.output.clone().unwrap_or_else(|| PathBuf::from("a.out"));
        match cli.emit {
            EmitKind::Executable => {
                let obj = output.with_extension("o");
                emit_file(&codegen, inkwell::targets::FileType::Object, &obj)?;
                link(&[obj.clone()], &output)?;
                std::fs::remove_file(&obj).ok();
            }
            EmitKind::Object => {
                emit_file(&codegen, inkwell::targets::FileType::Object, &output)?;
            }
            EmitKind::Asm => {
                emit_file(&codegen, inkwell::targets::FileType::Assembly, &output)?;
            }
            EmitKind::LlvmIr => {
                codegen.get_module().print_to_file(&output)?;
            }
        }
    }

    Ok(())
}
