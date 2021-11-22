use super::{BinaryExpr, ParseAst, Result, Rule, Term, Type, TypeOf, UnaryExpr};
use lazy_static::lazy_static;
use pest::{
    iterators::Pair,
    prec_climber::{Operator, PrecClimber},
};
use std::fmt::Display;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::Assoc::*;
        PrecClimber::new(vec![
            Operator::new(Rule::add, Left) | Operator::new(Rule::sub, Left),
            Operator::new(Rule::mul, Left) | Operator::new(Rule::div, Left),
        ])
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprInternal {
    BinaryExpr(BinaryExpr),
    UnaryExpr(UnaryExpr),
    Term(Term),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub internal: Box<ExprInternal>,
}

impl ParseAst for Expr {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::expr);

        PREC_CLIMBER.climb(
            token.into_inner(),
            |token: Pair<Rule>| match token.as_rule() {
                Rule::term => Ok(Expr {
                    internal: Box::new(ExprInternal::Term(ParseAst::parse(token)?)),
                }),
                Rule::unary => Ok(Expr {
                    internal: Box::new(ExprInternal::UnaryExpr(ParseAst::parse(token)?)),
                }),
                _ => unreachable!("{:?}", token),
            },
            |lhs: Result<Expr>, op: Pair<Rule>, rhs: Result<Expr>| {
                Ok(Expr {
                    internal: Box::new(ExprInternal::BinaryExpr(BinaryExpr::new(lhs?, op, rhs?)?)),
                })
            },
        )
    }
}

impl TypeOf for Expr {
    fn type_of(&self) -> Type {
        match self.internal.as_ref() {
            ExprInternal::BinaryExpr(v) => v as &dyn TypeOf,
            ExprInternal::UnaryExpr(v) => v as _,
            ExprInternal::Term(v) => v as _,
        }
        .type_of()
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.internal.as_ref() {
            ExprInternal::BinaryExpr(v) => v as &dyn Display,
            ExprInternal::UnaryExpr(v) => v as _,
            ExprInternal::Term(v) => v as _,
        }
        .fmt(f)
    }
}
