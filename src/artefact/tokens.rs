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
    /* /// `| ... |`
    ///
    /// takes the absolute value or something else in the future
    Absolute, */
}

pub struct Group<'s> {
    delimiter: Delimiter,
    tokens: Tokens<'s>,
}

pub enum Token<'s> {
    Ident(&'s str),
    LitInt(isize),
    LitStr(&'s str),
    Group(Group<'s>),

    __GroupDelimiter(Delimiter),

    /// `/*`
    __BlockCommentBegin,

    /// `*/`
    __BlockCommentEnd,

    /// `//`
    __InlineCommentBegin,

    /// `'\n'`
    __InlineCommentEnd,
}

/* pub struct Tokens {
    tokens:
} */

pub type Tokens<'s> = Vec<Token<'s>>;
