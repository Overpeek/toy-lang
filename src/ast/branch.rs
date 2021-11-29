use super::{Expr, ParseAst, Result, Rule, Scope, VisibleVars};
use crate::ast::{Error, Type, TypeOf};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct BranchInternal {
    pub test: Expr,
    pub on_true: Scope,
    pub on_false: Scope,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    pub internal: Box<BranchInternal>,

    ty: Option<Type>,
}

impl ParseAst for Branch {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::branch);

        let mut tokens = token.into_inner();
        let (test, test_span) = Expr::parse_spanned(tokens.next().unwrap(), vars)?;
        let on_true = Scope::parse(tokens.next().unwrap(), vars)?;
        let (on_false, on_false_span) = Scope::parse_spanned(tokens.next().unwrap(), vars)?;

        let ty_true = on_true.type_of_checked();
        let ty_false = on_false.type_of_checked();

        let ty = match (
            test.type_of_checked(),
            ty_true,
            ty_false,
            ty_true == ty_false,
        ) {
            (Some(Type::Bool), Some(_), Some(_), true) => ty_true,
            (Some(ty), Some(_), Some(_), true) => {
                return Err(Error::new_type_mismatch(test_span, Type::Bool, ty))
            }
            (Some(_), Some(ty_true), Some(ty_false), false) => {
                return Err(Error::new_type_mismatch(on_false_span, ty_true, ty_false))
            }
            (None, _, _, _) | (_, None, _, _) | (_, _, None, _) => None,
        };

        Ok(Self {
            internal: Box::new(BranchInternal {
                test,
                on_true,
                on_false,
            }),

            ty,
        })
    }
}

impl TypeOf for Branch {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "if {} {} else {}",
            self.internal.test, self.internal.on_true, self.internal.on_false
        )
    }
}
