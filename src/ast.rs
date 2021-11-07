use std::fmt::{Debug, Display};

use itertools::Itertools;
use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ToyLangParser;

pub enum Error {
    ParseError(pest::error::Error<Rule>),
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
            Error::ParseError(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Either<T, U> {
    A(T),
    B(U),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Add;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sub;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mul;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Div;

pub type Ident = String;
pub type Sign = Either<Add, Sub>;
pub type Call = Ident;

#[derive(Debug, Clone, PartialEq)]
pub enum Factor {
    F64(f64),
    I64(i64),
    Expr(Box<Expr>),
    Call(Call),
    Sign(Box<(Sign, Factor)>),
}

pub type Term = (Factor, Vec<(Either<Mul, Div>, Factor)>);
pub type Expr = (Term, Vec<(Sign, Term)>);
pub type Statement = Expr;
pub type Scope = Vec<Statement>;
pub type Function = (Ident, Scope);
pub type Module = Vec<Function>;

pub struct Ast {
    pub module: Module,
}

impl Ast {
    pub fn new(input: &str) -> Result<Self> {
        let mut tokens = ToyLangParser::parse(Rule::input, input).map_err(Error::ParseError)?;

        Ok(Self {
            module: Self::parse_module(tokens.next().unwrap()),
        })
    }

    fn parse_module(token: Pair<Rule>) -> Module {
        assert!(token.as_rule() == Rule::module);
        log::debug!("got mod");

        token.into_inner().map(Self::parse_function).collect()
    }

    fn parse_function(token: Pair<Rule>) -> Function {
        assert!(token.as_rule() == Rule::function);

        let mut tokens = token.into_inner();
        let name = tokens.next().unwrap().as_str().trim().to_string();
        log::debug!("got fn: '{}'", name);

        let scope = Self::parse_scope(tokens.next().unwrap());
        (name, scope)
    }

    fn parse_scope(token: Pair<Rule>) -> Scope {
        assert!(token.as_rule() == Rule::scope);
        log::debug!("got scope");

        token.into_inner().map(Self::parse_statement).collect()
    }

    fn parse_statement(token: Pair<Rule>) -> Statement {
        assert!(token.as_rule() == Rule::statement);
        log::debug!("got statement");

        Self::parse_expr(token.into_inner().next().unwrap())
    }

    fn parse_sign(token: Pair<Rule>) -> Sign {
        assert!(token.as_rule() == Rule::sign);
        log::debug!("got sign");

        match token.into_inner().next().unwrap().as_rule() {
            Rule::add => Sign::A(Add),
            Rule::sub => Sign::B(Sub),
            other => unreachable!("Sign cannot be: {:?}", other),
        }
    }

    fn parse_expr(token: Pair<Rule>) -> Expr {
        assert!(token.as_rule() == Rule::expr);
        log::debug!("got expr");

        let mut tokens = token.into_inner();

        let first = Self::parse_term(tokens.next().unwrap());
        let others = tokens
            .chunks(2)
            .into_iter()
            .map(|mut chunk| {
                let op = Self::parse_sign(chunk.next().unwrap());
                let next = Self::parse_term(chunk.next().unwrap());
                assert!(chunk.next().is_none());
                (op, next)
            })
            .collect();

        (first, others)
    }

    fn parse_mul_or_div(token: Pair<Rule>) -> Either<Mul, Div> {
        match token.as_rule() {
            Rule::mul => Either::A(Mul),
            Rule::div => Either::B(Div),
            other => unreachable!("Sign cannot be: {:?}", other),
        }
    }

    fn parse_term(token: Pair<Rule>) -> Term {
        assert!(token.as_rule() == Rule::term);
        log::debug!("got term");

        let mut tokens = token.into_inner();

        let first = Self::parse_factor(tokens.next().unwrap());
        let others = tokens
            .chunks(2)
            .into_iter()
            .map(|mut chunk| {
                let op = Self::parse_mul_or_div(chunk.next().unwrap());
                let next = Self::parse_factor(chunk.next().unwrap());
                assert!(chunk.next().is_none());
                (op, next)
            })
            .collect();

        (first, others)
    }

    fn parse_factor(token: Pair<Rule>) -> Factor {
        assert!(token.as_rule() == Rule::factor);
        log::debug!("got factor: '{}'", token.as_str().trim());

        let mut tokens = token.into_inner();
        let inner = tokens.next().unwrap();
        match inner.as_rule() {
            Rule::float => Self::parse_float(inner),
            Rule::int => Self::parse_int(inner),
            Rule::call => Self::parse_call(inner),
            Rule::expr => Factor::Expr(Box::new(Self::parse_expr(inner))),
            Rule::sign => {
                let sign = Self::parse_sign(inner);
                let rhs = Self::parse_factor(tokens.next().unwrap());
                Factor::Sign(Box::new((sign, rhs)))
            }
            other => unreachable!("{:?}", other),
        }
    }

    fn parse_float(token: Pair<Rule>) -> Factor {
        assert!(token.as_rule() == Rule::float);

        log::debug!("got float: '{}'", token.as_str().trim());
        Factor::F64(token.as_str().trim().parse().unwrap())
    }

    fn parse_int(token: Pair<Rule>) -> Factor {
        assert!(token.as_rule() == Rule::int);

        log::debug!("got int: '{}'", token.as_str().trim());
        Factor::I64(token.as_str().trim().parse().unwrap())
    }

    fn parse_call(token: Pair<Rule>) -> Factor {
        assert!(token.as_rule() == Rule::call);

        log::debug!("got call: '{}'", token.as_str());
        Factor::Call(token.into_inner().next().unwrap().as_str().to_string())
    }
}
