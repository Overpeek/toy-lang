use parse::lexer::run_lexer;

pub mod artefact;
pub mod parse;

const CODE: &str = r#"

/* comment */

// comment

fn main() -> f64 {
    3.0 + 0.141
}

"#;

fn main() {
    match run_lexer(CODE) {
        Ok(tokens) => {
            println!("got lexer tokens {:?}", tokens)
        }
        Err(err) => {
            println!("got lexer error: {:?}", err)
        }
    };
}
