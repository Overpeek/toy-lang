use pest::iterators::Pair;

use crate::ast::{Error, Type, TypeOf};

use super::{Expr, ParseAst, Result, Rule, Scope};
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
    ty: Type,
}

impl ParseAst for Branch {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::branch);

        let mut tokens = token.into_inner();
        let test: Expr = ParseAst::parse(tokens.next().unwrap())?;
        let on_true: Scope = ParseAst::parse(tokens.next().unwrap())?;
        let on_false: Scope = ParseAst::parse(tokens.next().unwrap())?;

        let ty = on_true.type_of();
        match (test.type_of(), ty == on_false.type_of()) {
            (Type::Bool, true) => {}
            (_, true) => return Err(Error::TypeMismatch(on_true.type_of(), on_false.type_of())),
            (ty, _) => return Err(Error::TypeMismatch(Type::Bool, ty)),
        }

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
    fn type_of(&self) -> Type {
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
