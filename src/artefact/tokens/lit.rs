use std::fmt::Display;

use super::{ToToken, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LitInt {
    pub value: isize,
}

impl LitInt {
    pub fn new(value: isize) -> Self {
        Self { value }
    }
}

impl ToToken for LitInt {
    fn to_token(self) -> Token {
        Token::Lit(Lit::LitInt(self))
    }
}

impl Display for LitInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LitFloat {
    pub value: f64,
}

impl LitFloat {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl ToToken for LitFloat {
    fn to_token(self) -> Token {
        Token::Lit(Lit::LitFloat(self))
    }
}

impl Display for LitFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LitStr {
    pub value: String,
}

impl LitStr {
    pub fn new<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            value: value.into(),
        }
    }
}

impl ToToken for LitStr {
    fn to_token(self) -> Token {
        Token::Lit(Lit::LitStr(self))
    }
}

impl Display for LitStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LitChar {
    pub value: char,
}

impl LitChar {
    pub fn new(value: char) -> Self {
        Self { value }
    }
}

impl ToToken for LitChar {
    fn to_token(self) -> Token {
        Token::Lit(Lit::LitChar(self))
    }
}

impl Display for LitChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    /// integer literals
    ///
    /// e.g. `42`, `-9`
    LitInt(LitInt),

    /// float literals
    ///
    /// e.g. `4.2`, `-9.0`
    LitFloat(LitFloat),

    /// string literals
    ///
    /// e.g. `"text"`
    LitStr(LitStr),

    /// character literals
    ///
    /// e.g. `'c'`, `' '`, `'\n'`
    LitChar(LitChar),
}

impl ToToken for Lit {
    fn to_token(self) -> Token {
        Token::Lit(self)
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::LitInt(v) => Display::fmt(v, f),
            Lit::LitFloat(v) => Display::fmt(v, f),
            Lit::LitStr(v) => Display::fmt(v, f),
            Lit::LitChar(v) => Display::fmt(v, f),
        }
    }
}
