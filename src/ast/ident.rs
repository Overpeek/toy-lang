use super::{ParseAst, Result, Rule};
use pest::iterators::Pair;

pub type Ident = String;

impl ParseAst for Ident {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::ident);
        Ok(token.as_str().into())
    }
}
