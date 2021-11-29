use super::{Expr, Ident, ParseAst, Result, Rule, Type, TypeOf, VisibleVars};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub name: Ident,
    pub expr: Expr,

    ty: Option<Type>,
}

impl ParseAst for Assign {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::assign);

        let mut tokens = token.into_inner();

        let name = Ident::parse(tokens.next().unwrap(), vars)?;
        let expr = Expr::parse(tokens.next().unwrap(), vars)?;
        let ty = expr.type_of();

        vars.push_var(name.value.as_str(), ty);

        Ok(Self {
            name,
            expr,
            ty: Some(ty),
        })
    }
}

impl TypeOf for Assign {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for Assign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {}", self.name, self.expr)
    }
}
