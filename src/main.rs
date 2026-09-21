mod codegen;
mod frontend;
mod jit;
mod repl;
pub mod stdlib;

use std::path::PathBuf;

use clap::Parser;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};

use crate::codegen::{Codegen, CodegenOptions};
use crate::frontend::ast::TopLevel;
use crate::jit::Jit;
use crate::repl::ReplInput;

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

    #[arg(long)]
    dry_run: bool,

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

fn run_repl_mode(repl_ctx: &mut ReplContext, cli: &Cli, opt_level: OptimizationLevel) -> Result<()> {
    repl::run_repl(|input| {
        match input {
            ReplInput::Item(item) => {
                let anon_name = repl_ctx
                    .codegen
                    .compile_top_level(&item)?
                    .unwrap_or(String::from("__anon_expr"));

                match opt_level {
                    OptimizationLevel::None => repl_ctx.codegen.optimize("default<O0>")?,
                    OptimizationLevel::Less => repl_ctx.codegen.optimize("default<O1>")?,
                    OptimizationLevel::Default => repl_ctx.codegen.optimize("default<O2>")?,
                    OptimizationLevel::Aggressive => repl_ctx.codegen.optimize("default<O3>")?,
                }

                if cli.ast {
                    eprintln!("=== AST === (stderr)");
                    eprintln!("{:#?}", item);
                }

                if cli.ir {
                    eprintln!("=== LLVM IR === (stderr)");
                    repl_ctx.codegen.get_module().print_to_stderr();
                }

                repl_ctx.jit.add_module(repl_ctx.codegen.take_module())?;

                if !cli.dry_run {
                    match item {
                        TopLevel::Expr(_) => {
                            if let Some(func) = repl_ctx.jit.lookup(anon_name.as_str()) {
                                let result = unsafe { func.call() };
                                println!("=== Evaluate === (stdout)");
                                println!("ans = {}", result);
                            } else {
                                eprintln!("=== Evaluate Failed === (stderr)");
                                eprintln!("cannot find {}", anon_name);
                            }
                        }
                        _ => {}
                    };
                }
            }

            ReplInput::Debug(cmd) => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                match parts.as_slice() {
                    [] | ["help"] | ["?"] => {
                        // TODO: help
                        eprintln!("TODO: help");
                    }

                    ["target"] => {
                        eprintln!("=== Target Machine === (stderr)");
                        eprintln!("CPU: {}", repl_ctx.codegen.get_target_machine().get_cpu());
                        eprintln!(
                            "Feature String: {:#?}",
                            repl_ctx.codegen.get_target_machine().get_feature_string()
                        );
                        eprintln!("Target: {:#?}", repl_ctx.codegen.get_target_machine().get_target());
                        eprintln!(
                            "Target Data:{:#?}",
                            repl_ctx.codegen.get_target_machine().get_target_data()
                        );
                        eprintln!("Triple: {}", repl_ctx.codegen.get_target_machine().get_triple());
                    }

                    ["ir"] => {
                        eprintln!("=== LLVM IR of current module === (stderr)");
                        repl_ctx.codegen.get_module().print_to_stderr();
                    }

                    ["protos"] => {
                        eprintln!("=== Function Prototypes === (stderr)");
                        for name in repl_ctx.codegen.get_function_proto_names() {
                            eprintln!("{}", name);
                        }
                    }

                    ["globals"] => {
                        eprintln!("=== Global Scope === (stderr)");
                        for name in repl_ctx.codegen.get_global_names() {
                            eprintln!("{}", name);
                        }
                    }

                    ["imported"] => {
                        eprintln!("=== Imported Markers === (stderr)");
                        for marker in repl_ctx.codegen.get_imported_markers() {
                            eprintln!("{:?}", marker);
                        }
                    }

                    ["scopes"] => {
                        eprintln!("=== Current Scope Depth === (stderr)");
                        eprintln!("scope depth: {}", repl_ctx.codegen.get_scope_depth());
                    }

                    ["anon"] => {
                        eprintln!("=== Current Anonymous Counter  === (stderr)");
                        eprintln!("anon counter: {}", repl_ctx.codegen.get_anon_counter());
                    }

                    ["options"] => {
                        eprintln!("=== ERR === (stderr)");
                        eprintln!("{:?}", repl_ctx.codegen.get_options());
                    }

                    _ => {
                        eprintln!("unknown debug command: {}", cmd);
                        eprintln!("try @help");
                    }
                }
            }
        };

        Ok(())
    })
}

fn compile_files(codegen: &mut Codegen, cli: &Cli) -> Result<()> {
    // TODO: compiler
    let _ = codegen;
    let _ = cli;
    todo!("compile files")
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let opt_level = match cli.optimize {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        3 => OptimizationLevel::Aggressive,
        _ => panic!("optimization level should be 0(None), 1(Less), 2(Default), 3(Aggressive)"),
    };

    eprintln!("Optimization level: {} => {:#?}", cli.optimize, opt_level);

    if cli.dry_run {
        eprintln!("WARNING: dry run");
    }

    let context = Context::create();

    let target_machine = create_target_machine(opt_level);

    let options = CodegenOptions {
        fast_math: cli.fast_math,
    };

    let mut codegen = Codegen::new(&context, "kaleidoscope_repl", target_machine, options);

    let mut jit = Jit::new(&context, codegen.get_target_machine(), opt_level).unwrap();

    if cli.files.is_empty() {
        codegen.compile_top_level(&TopLevel::Import("std".into()))?;

        if cli.ir {
            println!("=== Stdlib IR === (stderr)");
            codegen.get_module().print_to_stderr();
        }

        jit.add_module(codegen.take_module())?;

        println!("=== Repl === (stdout)");
        run_repl_mode(&mut ReplContext::new(codegen, jit), &cli, opt_level)?;
    } else {
        compile_files(&mut codegen, &cli)?;
    }

    Ok(())
}
