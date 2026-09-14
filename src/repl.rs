use chumsky::Parser;
use std::io::{self, BufRead, Write};

use crate::frontend::ast::TopLevel;
use crate::frontend::parser::create_top_level_parser;

use crate::Result;

pub fn run_repl<F>(mut on_item: F)
where
    F: FnMut(TopLevel) -> Result<()>,
{
    loop {
        print!("ready> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("input error: {}", e);
                break;
            }
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        let parser = create_top_level_parser();

        match parser.parse(input).into_result() {
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
}
