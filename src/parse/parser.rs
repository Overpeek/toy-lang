use crate::artefact::{ast::AST, tokens::Tokens};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnexpectedEOF,
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn run_parser(tokens: Tokens) -> AST {
    todo!()
}
