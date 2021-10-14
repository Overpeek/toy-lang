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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comment {
    /// `/* ... */`
    ///
    /// block comment, anything inside should be ignored
    Block,

    /// `// ... '\n|EOF'`
    ///
    /// inline comment, anything inside should be ignored
    /// ends with newline or end of file
    Inline,
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

    /// `| ... |`
    ///
    /// takes the absolute value or something else in the future
    // Absolute,

    /// `/* ... */` or `//`
    ///
    /// comments, anything inside should be ignored
    Comment(Comment),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// keywords or names
    ///
    /// e.g. `fn`
    Ident(String),

    /// integer literals
    ///
    /// e.g. `42`, `-9`
    LitInt(isize),

    /// float literals
    ///
    /// e.g. `4.2`, `-9.0`
    LitFloat(f64),

    /// string literals
    ///
    /// e.g. `"text"`
    LitStr(String),

    /// character literals
    ///
    /// e.g. `'c'`, `' '`, `'\n'`
    LitChar(char),

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

    /// `/*` or `*/`
    BlockComment(Side),

    /// `//` or `'\n'`
    InlineComment(Side),

    /// new line
    /// '\n'
    LF,

    /// end of file
    EOF,
}

pub type Tokens = Vec<Token>;
