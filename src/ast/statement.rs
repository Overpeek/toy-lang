use super::{Assign, Expr, ParseAst, Result, Rule, Type, TypeOf};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expr(Expr),
    Assign(Assign),
}

impl ParseAst for Statement {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        Ok(match token.as_rule() {
            Rule::expr => Statement::Expr(ParseAst::parse(token)?),
            Rule::assign => Statement::Assign(ParseAst::parse(token)?),
            _ => unreachable!("{:?}", token),
        })
    }
}

impl TypeOf for Statement {
    fn type_of(&self) -> Type {
        match self {
            Statement::Expr(v) => v as &dyn TypeOf,
            Statement::Assign(v) => v as _,
        }
        .type_of()
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Expr(v) => v as &dyn Display,
            Statement::Assign(v) => v as _,
        }
        .fmt(f)
    }
}
