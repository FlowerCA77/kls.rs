mod frontend;
mod repl;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        todo!()
    } else {
        repl::run_repl(|item| {
            println!("{:#?}", item);
            Ok(())
        });
    }
}
