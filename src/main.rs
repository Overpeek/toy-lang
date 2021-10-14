use colorful::Colorful;
use parse::lexer::run_lexer;
use std::io::{stdin, stdout, Write};

pub mod artefact;
pub mod parse;

fn main() {
    loop {
        let mut buf = String::new();
        print!("{} > ", env!("CARGO_CRATE_NAME"));
        stdout().flush().unwrap();
        stdin().read_line(&mut buf).unwrap();

        match run_lexer(&buf) {
            Ok(tokens) => {
                println!("lexer got tokens {:?}", tokens)
            }
            Err(err) => {
                println!("{}: {}", "error".red(), err)
            }
        }
    }
}
