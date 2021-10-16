use colorful::Colorful;
use parse::lexer::run_lexer;
use std::{
    env,
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
};

use crate::{
    artefact::tokens::SourceType, interpreter::run_interpreter, parse::parser::run_parser,
};

pub mod artefact;
pub mod interpreter;
pub mod parse;

fn run_code(code: &str, source_type: SourceType) {
    match run_lexer(code, source_type) {
        Ok(tokens) => {
            println!("{}: lexer got tokens {:?}", "debug".yellow(), tokens.tokens);

            match run_parser(&tokens) {
                Ok(ast) => {
                    println!("{}: parser got AST {:?}", "debug".yellow(), &ast);

                    let result = run_interpreter(&ast);
                    println!("{}: interpreter got result {:?}", "debug".yellow(), result);
                    println!("{} ", result);
                }
                Err(err) => {
                    println!("{}: {}", "error".red(), err)
                }
            }
        }
        Err(err) => {
            println!("{}: {}", "error".red(), err)
        }
    }
}

fn run_file(path: PathBuf) {
    match File::open(&path) {
        Ok(mut file) => {
            let mut buf = String::new();
            match file.read_to_string(&mut buf) {
                Ok(_) => run_code(&buf, SourceType::File(path)),
                Err(err) => println!("{}: {}", "error".red(), err),
            }
        }
        Err(err) => println!("{}: {}", "error".red(), err),
    }
}

fn cli() {
    loop {
        let mut buf = String::new();
        print!("{} > ", env!("CARGO_CRATE_NAME"));
        stdout().flush().unwrap();
        stdin().read_line(&mut buf).unwrap();

        run_code(&buf, SourceType::Stdin);
    }
}

fn main() {
    if let Some(file) = env::args().skip(1).next() {
        run_file(file.into())
    } else {
        cli()
    }
}
