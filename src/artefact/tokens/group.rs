use std::fmt::Display;

use super::{ToToken, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    ///  `( ... )`
    ///
    /// regular parentheses
    ///
    /// for order
    Parentheses,

    /// `{ ... }`
    ///
    /// curly braces
    ///
    /// for scopes
    Braces,

    /// `[ ... ]`
    ///
    /// square brackets
    ///
    /// for arrays
    Brackets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    pub delimiter: Delimiter,
    pub side: Side,
}

impl Group {
    pub fn new(delimiter: Delimiter, side: Side) -> Self {
        Self { delimiter, side }
    }
}

impl ToToken for Group {
    fn to_token(self) -> Token {
        Token::Group(self)
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match (self.delimiter, self.side) {
                (Delimiter::Parentheses, Side::Left) => '(',
                (Delimiter::Parentheses, Side::Right) => ')',
                (Delimiter::Braces, Side::Left) => '{',
                (Delimiter::Braces, Side::Right) => '}',
                (Delimiter::Brackets, Side::Left) => '[',
                (Delimiter::Brackets, Side::Right) => ']',
            }
        )
    }
}
