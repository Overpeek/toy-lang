use crate::{
    artefact::tokens::SourceType, interpreter::run_interpreter, parse::parser::run_parser,
};
use colorful::Colorful;
use parse::lexer::run_lexer;
use std::{
    env,
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
};

pub mod artefact;
pub mod compiler;
pub mod interpreter;
pub mod parse;

fn run_code(code: &str, source_type: SourceType) {
    // lexer run
    let tokens = match run_lexer(code, source_type) {
        Ok(tokens) => tokens,
        Err(err) => {
            return println!("{}: {}", "error".red(), err);
        }
    };

    // parser run
    let ast = match run_parser(&tokens) {
        Ok(ast) => ast,
        Err(err) => {
            return println!("{}: {}", "error".red(), err);
        }
    };

    // interpreter run
    let result = match run_interpreter(&ast) {
        Ok(result) => result,
        Err(err) => {
            return println!("{}: {}", "error".red(), err);
        }
    };

    println!("{}", result);
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

        let code = format!("fn main() {{{}}}", buf);

        run_code(&code, SourceType::Stdin);
    }
}

fn main() {
    env_logger::init();

    if let Some(file) = env::args().skip(1).next() {
        run_file(file.into())
    } else {
        cli()
    }
}
