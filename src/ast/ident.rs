use std::fmt::Display;

use super::{ParseAst, Result, Rule, VisibleVars};
use pest::iterators::Pair;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub value: String,
}

impl ParseAst for Ident {
    fn parse(token: Pair<Rule>, _: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::ident);

        Ok(Ident {
            value: token.as_str().into(),
        })
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
