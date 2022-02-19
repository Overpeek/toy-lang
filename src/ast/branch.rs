use super::{Ast, Expr, Result, Rule, Scope, VisibleVars};
use crate::ast::{match_rule, Error, Type, TypeOf};
use pest::{iterators::Pair, Span};
use std::fmt::Display;

//

#[derive(Debug, Clone, PartialEq)]
pub struct BranchInternal<'i> {
    pub test: Expr<'i>,
    pub on_true: Scope<'i>,
    pub on_false: Scope<'i>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Branch<'i> {
    pub internal: Box<BranchInternal<'i>>,

    span: Span<'i>,
    ty: Option<Type>,
}

//

impl<'i> Ast<'i> for Branch<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::branch)?;
        let mut tokens = token.into_inner();

        let test = Expr::parse(tokens.next().unwrap())?;
        let on_true = Scope::parse(tokens.next().unwrap())?;
        let on_false = Scope::parse(tokens.next().unwrap())?;

        Ok(Self {
            internal: Box::new(BranchInternal {
                test,
                on_true,
                on_false,
            }),

            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for Branch<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        self.internal.test.type_check(vars)?;
        self.internal.on_true.type_check(vars)?;
        self.internal.on_false.type_check(vars)?;

        let ty_test = self.internal.test.type_of();
        let ty_true = self.internal.on_true.type_of();
        let ty_false = self.internal.on_false.type_of();

        self.ty = match (ty_test, ty_true, ty_false, ty_true == ty_false) {
            (Type::Bool, _, _, true) => Some(ty_true),
            (ty, _, _, true) => {
                return Err(Error::new_type_mismatch(
                    self.internal.test.span(),
                    &Type::Bool,
                    &ty,
                ))
            }
            (_, Type::Unresolved, ty_false, false) => Some(ty_false),
            (_, ty_true, Type::Unresolved, false) => Some(ty_true),
            (_, ty_true, ty_false, false) => {
                return Err(Error::new_type_mismatch(
                    self.internal.on_false.span(),
                    &ty_true,
                    &ty_false,
                ))
            }
        };

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for Branch<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "if {} {} else {}",
            self.internal.test, self.internal.on_true, self.internal.on_false
        )
    }
}
