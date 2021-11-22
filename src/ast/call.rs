use super::{Ident, ParseAst, Result, Rule, Type, TypeOf};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: Ident,
}

impl ParseAst for Call {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::call);

        Ok(Self {
            name: ParseAst::parse_single(token.into_inner())?,
        })
    }
}

impl TypeOf for Call {
    fn type_of(&self) -> Type {
        Type::Unit // TODO:
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}()", self.name)
    }
}
