use pest::{
    error::{Error as PestError, ErrorVariant},
    iterators::{Pair, Pairs},
    Parser,
};
use std::fmt::{Debug, Display};
use std::{io, path::Path};

pub use self::access::*;
pub use self::assign::*;
pub use self::binary::*;
pub use self::branch::*;
pub use self::call::*;
pub use self::expr::*;
pub use self::function::*;
pub use self::ident::*;
pub use self::module::*;
pub use self::r#type::*;
pub use self::scope::*;
pub use self::statement::*;
pub use self::term::*;
pub use self::unary::*;

pub mod access;
pub mod assign;
pub mod binary;
pub mod branch;
pub mod call;
pub mod expr;
pub mod function;
pub mod ident;
pub mod module;
pub mod scope;
pub mod statement;
pub mod term;
pub mod r#type;
pub mod unary;

// ------
// Parser
// ------

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ToyLangParser;

pub fn parse(input: &str) -> Result<Module> {
    let mut tokens = ToyLangParser::parse(Rule::input, input).map_err(Error::ParseError)?;
    ParseAst::parse(tokens.next().unwrap())
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Module> {
    let file = std::fs::read_to_string("tests/script.tls").unwrap();
    match parse(&file) {
        Err(Error::ParseError(err)) => Err(Error::ParseError(
            err.with_path(path.as_ref().to_string_lossy().as_ref()),
        )),
        other => other,
    }
}

// ----------
// Error type
// ----------

pub enum Error {
    ParseError(PestError<Rule>),
    LeftoverTokens(PestError<Rule>),
    IoError(io::Error),
    InvalidBinaryOp(Type, BinaryOp, Type),
    InvalidUnaryOp(UnaryOp, Type),
    TypeMismatch(Type, Type),
}
pub type Result<T> = ::std::result::Result<T, Error>;

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(err) | Error::LeftoverTokens(err) => write!(f, "{}", err),
            Error::IoError(err) => write!(f, "{}", err),
            Error::InvalidBinaryOp(lhs, op, rhs) => write!(
                f,
                "the binary operator '{}' not implemented for lhs: '{}' and rhs: '{}'",
                op, lhs, rhs
            ),
            Error::InvalidUnaryOp(op, rhs) => {
                write!(
                    f,
                    "the unary operator '{}' not implemented for: '{}'",
                    op, rhs
                )
            }
            Error::TypeMismatch(a, b) => write!(f, "expected type: '{}' but got: '{}'", a, b),
        }
    }
}

// -----------
// Parse trait
// -----------

trait ParseAst
where
    Self: Sized,
{
    fn parse(token: Pair<Rule>) -> Result<Self>;

    fn parse_single(mut tokens: Pairs<Rule>) -> Result<Self> {
        let result = Self::parse(tokens.next().unwrap());
        if let Some(token) = tokens.next() {
            Err(Error::LeftoverTokens(PestError::new_from_span(
                ErrorVariant::CustomError {
                    message: "unexpected token".into(),
                },
                token.as_span(),
            )))
        } else {
            result
        }
    }

    fn parse_multiple(tokens: Pairs<Rule>) -> Result<Vec<Self>> {
        tokens.map(Self::parse).collect()
    }
}

trait TypeOf {
    fn type_of(&self) -> Type;
}
