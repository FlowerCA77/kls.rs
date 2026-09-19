use chumsky::Parser;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

use crate::Result;
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

pub fn run_repl<F>(mut on_item: F) -> Result<()>
where F: FnMut(TopLevel) -> Result<()> {
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

                match create_top_level_parser().parse(input).into_result() {
                    Ok(item) => {
                        if let Err(e) = on_item(item) {
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
