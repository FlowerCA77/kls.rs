mod codegen;
mod frontend;
mod repl;

use std::path::PathBuf;

use crate::codegen::{Codegen, CodegenOptions};

use clap::Parser;
use inkwell::{
    OptimizationLevel,
    context::Context,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    files: Vec<PathBuf>,

    #[arg(long)]
    ast: bool,

    #[arg(long)]
    ir: bool,

    #[arg(long)]
    fast_math: bool,
}

fn create_target_machine() -> TargetMachine {
    Target::initialize_all(&InitializationConfig::default());

    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();

    target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .unwrap()
}

fn run_repl_mode(codegen: &mut Codegen, cli: &Cli) {
    repl::run_repl(|item| {
        if cli.ast {
            println!("{:#?}", item);
        }
        if cli.ir {
            codegen.compile_top_level(&item)?;
            codegen.optimize("default<O2>")?;
            codegen.get_module().print_to_stderr();
        }
        Ok(())
    });
}

fn compile_files(codegen: &mut Codegen, cli: &Cli) {
    // TODO: compiler
    let _ = codegen;
    let _ = cli;
    todo!("compile files")
}

fn main() {
    let cli = Cli::parse();

    let context = Context::create();

    let target_machine = create_target_machine();

    let options = CodegenOptions {
        fast_math: cli.fast_math,
    };

    let mut codegen = Codegen::new(&context, "kaleidoscope_repl", target_machine, options);

    if cli.files.is_empty() {
        run_repl_mode(&mut codegen, &cli);
    } else {
        compile_files(&mut codegen, &cli);
    }
}
