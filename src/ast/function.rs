use crate::ast::TypeOf;

use super::{Ident, ParseAst, Result, Rule, Scope, Type, VisibleVars};
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

    ty: Option<Type>,
}

impl ParseAst for Function {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::function);

        let mut tokens = token.into_inner();

        let internal = Box::new(FunctionInternal {
            name: ParseAst::parse(tokens.next().unwrap(), vars)?,
            scope: ParseAst::parse(tokens.next().unwrap(), vars)?,
        });
        let ty = internal.scope.type_of();

        vars.push_var(internal.name.value.as_str(), ty);

        Ok(Self {
            internal,
            ty: Some(ty),
        })
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}() {}", self.internal.name, self.internal.scope)
    }
}
