use std::fmt::Display;

use super::{ToToken, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    Let,
    If,
    Else,
    True,
    False,
}

impl ToToken for Keyword {
    fn to_token(self) -> super::Token {
        Token::Keyword(self)
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Keyword::Fn => "fn",
                Keyword::Let => "let",
                Keyword::If => "if",
                Keyword::Else => "else",
                Keyword::True => "true",
                Keyword::False => "false",
            }
        )
    }
}
