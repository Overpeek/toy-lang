use super::{match_rule, Ast, Expr, GenericSolver, Ident, Result, Rule, Type, TypeOf, VisibleVars};
use pest::{iterators::Pair, Span};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Assign<'i> {
    pub name: Ident<'i>,
    pub expr: Expr<'i>,

    span: Span<'i>,
    ty: Option<Type>,
}

impl<'i> Ast<'i> for Assign<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::assign)?;
        let mut tokens = token.into_inner();

        let name = Ident::parse(tokens.next().unwrap())?;
        let expr = Expr::parse(tokens.next().unwrap())?;

        Ok(Self {
            name,
            expr,

            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for Assign<'i> {
    fn type_check_impl(
        &mut self,
        vars: &mut VisibleVars,
        solver: &mut GenericSolver<'i>,
    ) -> Result<()> {
        self.expr.type_check(vars, solver)?;
        let ty = self.expr.type_of();
        self.ty = Some(ty);
        vars.push_var(self.name.value.as_str(), ty);

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for Assign<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {}", self.name, self.expr)
    }
}
