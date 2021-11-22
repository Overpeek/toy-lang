use super::{Ident, ParseAst, Result, Rule, Type, TypeOf};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Access {
    pub name: Ident,
}

impl ParseAst for Access {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::access);

        Ok(Self {
            name: ParseAst::parse_single(token.into_inner())?,
        })
    }
}

impl TypeOf for Access {
    fn type_of(&self) -> Type {
        Type::Unit // TODO
    }
}

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}
