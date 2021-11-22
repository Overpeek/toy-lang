use super::{Expr, Ident, ParseAst, Result, Rule, Type, TypeOf};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub name: Ident,
    pub expr: Expr,
}

impl ParseAst for Assign {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::assign);

        let mut tokens = token.into_inner();
        Ok(Self {
            name: ParseAst::parse(tokens.next().unwrap())?,
            expr: ParseAst::parse(tokens.next().unwrap())?,
        })
    }
}

impl TypeOf for Assign {
    fn type_of(&self) -> Type {
        Type::Unit
    }
}

impl Display for Assign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {}", self.name, self.expr)
    }
}
