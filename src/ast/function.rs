use super::{Ident, ParseAst, Result, Rule, Scope};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionInternal {
    pub name: Ident,
    pub scope: Scope,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub internal: Box<FunctionInternal>,
}

impl ParseAst for Function {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::function);

        let mut tokens = token.into_inner();
        Ok(Self {
            internal: Box::new(FunctionInternal {
                name: ParseAst::parse(tokens.next().unwrap())?,
                scope: ParseAst::parse(tokens.next().unwrap())?,
            }),
        })
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}() {}", self.internal.name, self.internal.scope)
    }
}
