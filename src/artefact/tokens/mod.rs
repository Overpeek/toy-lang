use std::fmt::Display;

pub mod group;
pub mod keyword;
pub mod lit;
pub mod op;
pub mod span;
pub use group::*;
pub use keyword::*;
pub use lit::*;
pub use op::*;
pub use span::*;

pub trait ToToken: ToString {
    fn to_token(self) -> Token;
    fn to_spanned_token(self, span: Span) -> SpannedToken
    where
        Self: Sized,
    {
        SpannedToken::new(self.to_token(), span)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// identifiers
    ///
    /// e.g. `var_a`
    Ident(String),

    /// keywords
    ///
    /// e.g. `fn`
    Keyword(Keyword),

    /// literals
    ///
    /// e.g. `0.4` or `"xyz"`
    Lit(Lit),

    /// group begin or end
    Group(Group),

    /// math operators
    Operator(Operator),

    /// '.'
    Dot,

    /// ','
    Comma,

    /// ':'
    Colon,

    /// ';'
    Semicolon,

    /// `->`
    Arrow,

    /// =
    ///
    /// not to be confused with Operator::Eq
    Assign,

    /// new line
    /// '\n'
    LF,

    /// end of file
    EOF,
}

impl Token {
    pub fn to_spanned(self, span: Span) -> SpannedToken {
        SpannedToken::new(self, span)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(v) => Display::fmt(v, f),
            Token::Keyword(v) => Display::fmt(v, f),
            Token::Lit(v) => Display::fmt(v, f),
            Token::Group(v) => Display::fmt(v, f),
            Token::Operator(v) => Display::fmt(v, f),
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Arrow => write!(f, "->"),
            Token::Assign => write!(f, "="),
            Token::LF => write!(f, "\n"),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub value: Token,
    pub span: Span,
}

impl SpannedToken {
    pub fn new(value: Token, span: Span) -> Self {
        Self { value, span }
    }
}

pub struct Tokens {
    pub code: Vec<char>,
    pub source_type: SourceType,
    pub tokens: Vec<SpannedToken>,
}

impl Tokens {
    pub fn new(code: &str, source_type: SourceType) -> Self {
        Self {
            code: code.chars().collect(),
            source_type,
            tokens: Vec::new(),
        }
    }

    pub fn push(&mut self, token: SpannedToken) {
        self.tokens.push(token)
    }
}
