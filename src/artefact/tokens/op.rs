use std::fmt::Display;

use super::{ToToken, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// `+`
    Add,

    /// `-`
    Sub,

    /// `*`
    Mul,

    /// `/`
    Div,

    /// `==`
    ///
    /// not to be confused with Token::Assign
    Eq,

    /// `>=`
    Ge,

    /// `>`
    Gt,

    /// `<=`
    Le,

    /// `<`
    Lt,
}

impl ToToken for Operator {
    fn to_token(self) -> Token {
        Token::Operator(self)
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Eq => write!(f, "=="),
            Operator::Ge => write!(f, ">="),
            Operator::Gt => write!(f, ">"),
            Operator::Le => write!(f, "<="),
            Operator::Lt => write!(f, "<"),
        }
    }
}
