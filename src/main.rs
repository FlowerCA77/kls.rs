mod codegen;
mod frontend;
mod repl;

use crate::codegen::codegen::Codegen;
use inkwell::context::Context;
use std::env;

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && !std::env::args().any(|a| a.starts_with("--")) {
        todo!("compile files")
    } else {
        let show_ast = env::args().any(|a| a == "--ast");
        let show_ir = env::args().any(|a| a == "--ir");

        let context = Context::create();
        let mut codegen = Codegen::new(&context, "kaleidoscope_repl");

        repl::run_repl(|item| {
            if show_ast {
                println!("{:#?}", item);
            }
            if show_ir {
                codegen.compile_top_level(&item)?;
                codegen.get_module().print_to_stderr();
            }
            Ok(())
        });
    }
}
