use super::{match_rule, Ast, Result, Rule};
use pest::{iterators::Pair, Span};
use std::fmt::Display;

//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident<'i> {
    pub value: String,
    span: Span<'i>,
}

//

impl<'i> Ast<'i> for Ident<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::ident)?;

        Ok(Ident {
            value: token.as_str().into(),
            span,
        })
    }
}

impl<'i> Display for Ident<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<'i> Ident<'i> {
    pub fn new(s: &'i str) -> Self {
        Self {
            value: s.into(),
            span: Span::new(s, 0, s.len()).unwrap(),
        }
    }

    pub fn from(value: String, span: Span<'i>) -> Self {
        Self { value, span }
    }
}
