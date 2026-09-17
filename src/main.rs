mod codegen;
mod frontend;
mod jit;
mod repl;

use std::path::PathBuf;

use crate::{
    codegen::{Codegen, CodegenOptions},
    frontend::ast::TopLevel,
    jit::Jit,
};

use clap::Parser;
use inkwell::{
    OptimizationLevel,
    context::Context,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct ReplContext<'ctx> {
    codegen: Codegen<'ctx>,
    jit: Jit<'ctx>,
}

impl<'ctx> ReplContext<'ctx> {
    fn new(codegen: Codegen<'ctx>, jit: Jit<'ctx>) -> Self {
        Self { codegen, jit }
    }
}

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

    #[arg(short, long, default_value_t = 2)]
    optimize: u8,
}

fn create_target_machine(opt_level: OptimizationLevel) -> TargetMachine {
    Target::initialize_all(&InitializationConfig::default());

    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();

    target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            opt_level,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .unwrap()
}

fn run_repl_mode(repl_ctx: &mut ReplContext, cli: &Cli, opt_level: OptimizationLevel) {
    repl::run_repl(|item| {
        repl_ctx.codegen.compile_top_level(&item)?;
        match opt_level {
            OptimizationLevel::None => repl_ctx.codegen.optimize("default<O0>")?,
            OptimizationLevel::Less => repl_ctx.codegen.optimize("default<O1>")?,
            OptimizationLevel::Default => repl_ctx.codegen.optimize("default<O2>")?,
            OptimizationLevel::Aggressive => repl_ctx.codegen.optimize("default<O3>")?,
        }

        repl_ctx.jit.add_module(repl_ctx.codegen.take_module())?;

        match item {
            TopLevel::Expr(_) => {
                if let Some(func) = repl_ctx.jit.lookup("__anon_expr") {
                    let result = unsafe { func.call() };
                    println!("{}", result);
                }
            }
            _ => {}
        };

        if cli.ast {
            println!("=========== AST ===========");
            println!("{:#?}", item);
        }

        if cli.ir {
            println!("========= LLVM IR =========");
            repl_ctx.codegen.get_module().print_to_stderr();
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

    let opt_level = match cli.optimize {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        3 => OptimizationLevel::Aggressive,
        _ => panic!("optimization level should be 0(None), 1(Less), 2(Default), 3(Aggressive)"),
    };

    eprintln!("Optimization level: {} => {:#?}", cli.optimize, opt_level);

    let context = Context::create();

    let target_machine = create_target_machine(opt_level);

    let options = CodegenOptions {
        fast_math: cli.fast_math,
    };

    let mut codegen = Codegen::new(&context, "kaleidoscope_repl", target_machine, options);

    let jit = Jit::new(&context, codegen.get_target_machine(), opt_level).unwrap();

    if cli.files.is_empty() {
        run_repl_mode(&mut ReplContext::new(codegen, jit), &cli, opt_level);
    } else {
        compile_files(&mut codegen, &cli);
    }
}
