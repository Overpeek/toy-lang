use pest::{
    error::{Error as PestError, ErrorVariant},
    iterators::{Pair, Pairs},
    Parser, Span,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};
use std::{hash::Hash, path::Path};

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
    let mut tokens = ToyLangParser::parse(Rule::input, input).map_err(Error::new_pest)?;
    let mut globals = VisibleVars::new();
    ParseAst::parse(tokens.next().unwrap(), &mut globals)
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> ParseFileResult {
    let file = std::fs::read_to_string(path).map_err(ParseFileError::IoError)?;
    parse(&file).map_err(ParseFileError::ParseError)
}

//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleVars {
    vars: Vec<HashMap<String, Type>>,
}

impl Default for VisibleVars {
    fn default() -> Self {
        Self {
            vars: vec![Default::default()],
        }
    }
}

impl VisibleVars {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_var(&mut self, name: &str, ty: Type) {
        self.vars
            .last_mut()
            .as_mut()
            .unwrap()
            .insert(name.into(), ty);
    }

    pub fn get_var(&self, name: &str) -> Option<Type> {
        self.vars
            .iter()
            .rev()
            .find_map(|map| map.get(name))
            .cloned()
    }

    pub fn push(&mut self) {
        self.vars.push(Default::default())
    }

    pub fn pop(&mut self) {
        self.vars.pop();
    }
}

// ----------
// Error type
// ----------

pub enum ParseFileError {
    IoError(std::io::Error),
    ParseError(Error),
}

pub type ParseFileResult = ::std::result::Result<Module, ParseFileError>;

impl Debug for ParseFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for ParseFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(err) => Display::fmt(err, f),
            Self::IoError(err) => Display::fmt(err, f),
        }
    }
}

pub struct Error {
    error: PestError<Rule>,
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn new_spanned<S: Into<String>>(span: Span, message: S) -> Self {
        Self {
            error: PestError::new_from_span(
                ErrorVariant::CustomError {
                    message: message.into(),
                },
                span,
            ),
        }
    }

    pub fn new_pest(error: PestError<Rule>) -> Self {
        Self { error }
    }

    pub fn new_leftover_tokens(span: Span, token: Pair<Rule>) -> Self {
        Self::new_spanned(span, format!("unexpected token: '{}'", token.as_str()))
    }

    pub fn new_invalid_unary_op(span: Span, op: UnaryOp, ty: Type) -> Self {
        Self::new_spanned(
            span,
            format!(
                "unary operator: '{}' cannot be applied to type: '{}'",
                op, ty
            ),
        )
    }

    pub fn new_invalid_binary_op(span: Span, lhs: Type, op: BinaryOp, rhs: Type) -> Self {
        Self::new_spanned(
            span,
            format!(
                "binary operator: '{}' cannot be applied to lhs: '{}' and rhs: '{}'",
                op, lhs, rhs
            ),
        )
    }

    pub fn new_type_mismatch(span: Span, expect: Type, got: Type) -> Self {
        Self::new_spanned(
            span,
            format!("expected type: '{}' but got: '{}'", expect, got),
        )
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

// -----------
// Parse trait
// -----------

trait ParseAst
where
    Self: Sized,
{
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self>;

    fn parse_spanned<'i>(
        token: Pair<'i, Rule>,
        vars: &mut VisibleVars,
    ) -> Result<(Self, Span<'i>)> {
        let span = token.as_span();
        Ok((Self::parse(token, vars)?, span))
    }

    fn parse_single(mut tokens: Pairs<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        let result = Self::parse(tokens.next().unwrap(), vars);
        if let Some(token) = tokens.next() {
            Err(Error::new_leftover_tokens(token.as_span(), token))
        } else {
            result
        }
    }

    fn parse_multiple(tokens: Pairs<Rule>, vars: &mut VisibleVars) -> Result<Vec<Self>> {
        tokens
            .into_iter()
            .map(|token| Self::parse(token, vars))
            .collect()
    }
}

trait TypeOf {
    fn type_of_checked(&self) -> Option<Type>;

    fn type_of(&self) -> Type {
        self.type_of_checked().expect("Wasn't type checked")
    }
}

trait Typed<T> {
    fn typed(self) -> T;
}
