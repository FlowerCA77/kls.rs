use std::process::Command;

use chumsky::Parser;
use inkwell::OptimizationLevel;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

use crate::Result;
use crate::backend::jit::Jit;
use crate::cli::Cli;
use crate::codegen::Codegen;
use crate::frontend::ast::TopLevel;
use crate::frontend::parser::create_top_level_parser;

#[derive(Completer, Helper, Highlighter, Hinter)]
struct KaleidoscopeHelper;

impl Validator for KaleidoscopeHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        let mut depth: i32 = 0;
        for c in input.chars() {
            match c {
                '{' | '(' => depth += 1,
                '}' | ')' => depth -= 1,
                _ => {}
            }
        }

        if depth > 0 {
            Ok(ValidationResult::Incomplete)
        } else if depth < 0 {
            Ok(ValidationResult::Invalid(Some("Mismatched brackets".to_string())))
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

pub struct ReplContext<'ctx> {
    codegen: Codegen<'ctx>,
    jit: Jit<'ctx>,
}

impl<'ctx> ReplContext<'ctx> {
    pub fn new(codegen: Codegen<'ctx>, jit: Jit<'ctx>) -> Self {
        Self { codegen, jit }
    }
}

enum ReplInput {
    Item(TopLevel),
    Debug(String),
    Shcmd(String),
}

fn run_prompt<F>(mut handler: F) -> Result<()>
where
    F: FnMut(ReplInput) -> Result<()>,
{
    let mut rl = Editor::new()?;
    rl.set_helper(Some(KaleidoscopeHelper));

    let mut interrupted = false;

    let mut retval = Ok(());

    loop {
        match rl.readline("ready> ") {
            Ok(line) => {
                interrupted = false;
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                rl.add_history_entry(input)?;

                if let Some(cmd) = input.strip_prefix('@') {
                    if let Err(e) = handler(ReplInput::Debug(cmd.to_string())) {
                        eprintln!("debug error: {}", e);
                    }
                    continue;
                }

                if let Some(cmd) = input.strip_prefix('!') {
                    if let Err(e) = handler(ReplInput::Shcmd(cmd.to_string())) {
                        eprintln!("debug error: {}", e);
                    }
                    continue;
                }

                match create_top_level_parser().parse(input).into_result() {
                    Ok(item) => {
                        if let Err(e) = handler(ReplInput::Item(item)) {
                            eprintln!("error: {}", e);
                        }
                    }
                    Err(errors) => {
                        for e in errors {
                            eprintln!("parse error: {:?}", e);
                        }
                    }
                }
            }

            Err(ReadlineError::Interrupted) => {
                if interrupted {
                    println!();
                    retval = Err("repl: interrupted".into());
                    break;
                }
                interrupted = true;
                eprintln!("Press Ctrl-C again to exit");
            }

            Err(ReadlineError::Eof) => {
                println!();
                break;
            }

            Err(e) => {
                eprintln!("input error: {}", e);
                retval = Err(format!("repl: got input error: {}", e).into());
                break;
            }
        }
    }

    retval
}

pub fn run_repl(repl_ctx: &mut ReplContext, cli: &Cli, opt_level: OptimizationLevel) -> Result<()> {
    run_prompt(|input| {
        match input {
            ReplInput::Item(item) => {
                let anon_name = repl_ctx
                    .codegen
                    .compile_top_level(&item)?
                    .unwrap_or(String::from("__anon_expr"));

                repl_ctx
                    .codegen
                    .optimize(Cli::optimization_level_string(opt_level).as_str())?;

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

            ReplInput::Shcmd(shcmd) => {
                eprintln!("=== Shell Command Output ===");
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
                let status = Command::new(shell.clone()).arg("-c").arg(shcmd).status()?;
                eprintln!("=== Shell Command Info === (stderr)");
                eprintln!("Shell: {}", shell);
                eprintln!("Status: {}", status);
            }
        };

        Ok(())
    })
}
