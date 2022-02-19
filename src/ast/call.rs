use super::{Ast, Expr, Function, Ident, Result, Rule, Type, TypeOf, VisibleVars};
use crate::ast::match_rule;
use pest::{iterators::Pair, Span};
use std::fmt::Display;

//

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'i> {
    pub name: Ident<'i>,
    pub args: Vec<Expr<'i>>,

    span: Span<'i>,
    ty: Option<Type>,
}

//

impl<'i> Ast<'i> for Call<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::call)?;
        let mut tokens = token.into_inner();

        let name: Ident = Ast::parse(tokens.next().unwrap())?;
        let args = tokens
            .next()
            .map(|token| Ast::parse_multiple(token.into_inner()))
            .unwrap_or_else(|| Ok(vec![]))?;

        Ok(Self {
            name,
            args,
            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for Call<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        for arg in self.args.iter_mut() {
            arg.type_check(vars)?;
        }

        let fn_name = self.name.value.as_str();
        let sig: Box<[Type]> = self.args.iter().map(|arg| arg.type_of()).collect();

        let ty = if let Ok(ty) = vars.get_fn_ty(self.span(), fn_name, &sig) {
            ty
        } else {
            let f = Function::new(vars, self.span(), fn_name, &sig)?;
            let ty = f.type_of();
            vars.push_fn(fn_name, &sig, f);
            ty
        };

        self.ty = Some(ty);

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for Call<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}()", self.name)
    }
}
