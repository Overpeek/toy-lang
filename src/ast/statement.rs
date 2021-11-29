use super::{Assign, Expr, ParseAst, Result, Rule, Type, TypeOf, VisibleVars};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementInternal {
    Expr(Expr),
    Assign(Assign),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub internal: Box<StatementInternal>,

    ty: Option<Type>,
}

impl ParseAst for Statement {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        let internal = Box::new(match token.as_rule() {
            Rule::expr => StatementInternal::Expr(ParseAst::parse(token, vars)?),
            Rule::assign => StatementInternal::Assign(ParseAst::parse(token, vars)?),
            _ => unreachable!("{:?}", token),
        });
        let ty = match internal.as_ref() {
            StatementInternal::Expr(v) => v as &dyn TypeOf,
            StatementInternal::Assign(v) => v as _,
        }
        .type_of_checked();

        Ok(Statement { internal, ty })
    }
}

impl TypeOf for Statement {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.internal.as_ref() {
            StatementInternal::Expr(v) => v as &dyn Display,
            StatementInternal::Assign(v) => v as _,
        }
        .fmt(f)
    }
}
