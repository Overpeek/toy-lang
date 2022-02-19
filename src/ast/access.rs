use super::{Ast, Error, Ident, Result, Rule, Type, TypeOf, VisibleVars};
use crate::ast::match_rule;
use pest::{iterators::Pair, Span};
use std::fmt::Display;

//

#[derive(Debug, Clone, PartialEq)]
pub struct Access<'i> {
    pub name: Ident<'i>,

    span: Span<'i>,
    ty: Option<Type>,
}

//

impl<'i> Ast<'i> for Access<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::access)?;
        let name = Ident::parse_single(token.into_inner())?;

        Ok(Self {
            name,
            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for Access<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        let name = self.name.value.as_str();
        self.ty = match vars.get_var(name) {
            Some(ty) => Some(ty),
            None => return Err(Error::new_var_not_found(self.span(), name)),
        };
        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for Access<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}
