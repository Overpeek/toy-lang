use std::{fmt::Display, ops::Range, path::PathBuf};

pub trait ToToken {
    fn to_token(self) -> Token;
    fn to_spanned_token(self, span: Span) -> SpannedToken
    where
        Self: Sized,
    {
        SpannedToken::new(self.to_token(), span)
    }
}

/* impl<T> Into<Token> for T
where
    T: ToToken,
{
    fn into(self) -> Token {
        self.to_token()
    }
} */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// +
    Add,

    /// -
    Sub,

    /// *
    Mul,

    /// /
    Div,
}

impl ToToken for Operator {
    fn to_token(self) -> Token {
        Token::Operator(self)
    }
}

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
pub enum Keyword {
    Fn,
    Let,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span(Range<usize>);

impl Span {
    pub fn new(range: Range<usize>) -> Self {
        Self { 0: range }
    }

    pub fn make_error_span(&self, code: &Vec<char>, source_type: SourceType) -> ErrorSpan {
        let range = self.range();

        let before = code[..range.start]
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, c)| (range.start - i, c))
            .find(|&(_, &c)| c == '\n')
            .map(|(i, _)| i);

        let after = code[range.end..]
            .into_iter()
            .enumerate()
            .map(|(i, c)| (range.end + i, c))
            .find(|&(_, &c)| c == '\n')
            .map(|(i, _)| i);

        let line_span = match (before, after) {
            (Some(before), Some(after)) => Span::new(before..after),
            (None, Some(after)) => Span::new(0..after),
            (Some(before), None) => Span::new(before..code.len()),
            (None, None) => Span::new(0..code.len()),
        };

        let mut row = 0;
        let mut col = 0;
        for c in code[..range.start].into_iter() {
            match c {
                '\n' => {
                    col = 0;
                    row += 1
                }
                _ => col += 1,
            }
        }

        let code_row = code[line_span.range()].into_iter().collect();

        ErrorSpan {
            error_span: self.clone(),
            line_span,

            row,
            col,

            code_row,
            source_type,
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    Stdin,
    File(PathBuf),
}

impl Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Stdin => write!(f, "<stdin>"),
            SourceType::File(path) => write!(f, "{}", path.as_os_str().to_string_lossy()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorSpan {
    error_span: Span,
    line_span: Span,

    row: usize,
    col: usize,

    code_row: String,
    source_type: SourceType,
}

impl Display for ErrorSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_range = self.error_span.range();

        write!(
            f,
            "  at {}:{}:{}\n\n  {}\n  {}{} ",
            self.source_type,
            self.row + 1,
            self.col + 1,
            self.code_row,
            " ".repeat(self.col),
            "^".repeat(error_range.len())
        )
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
    Group(Delimiter, Side),

    /// math operators
    Operator(Operator),

    /// '.'
    Dot,

    /// ','
    Comma,

    /// `->`
    Arrow,

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
