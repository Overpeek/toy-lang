use super::{BinaryExpr, ParseAst, Result, Rule, Term, Type, TypeOf, UnaryExpr, VisibleVars};
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
            //
            Operator::new(Rule::add, Left) | Operator::new(Rule::sub, Left),
            Operator::new(Rule::mul, Left) | Operator::new(Rule::div, Left),

            //
            Operator::new(Rule::eq, Left)
                | Operator::new(Rule::ne, Left)
                | Operator::new(Rule::gt, Left)
                | Operator::new(Rule::ge, Left)
                | Operator::new(Rule::lt, Left)
                | Operator::new(Rule::le, Left),

            //
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

    ty: Option<Type>,
}

impl ParseAst for Expr {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::expr);

        let span = token.as_span();

        PREC_CLIMBER.climb(
            token.into_inner(),
            |token: Pair<Rule>| match token.as_rule() {
                Rule::term => {
                    let inner = Term::parse(token, vars)?;
                    let ty = inner.type_of_checked();

                    Ok(Expr {
                        internal: Box::new(ExprInternal::Term(inner)),
                        ty,
                    })
                }
                Rule::unary => {
                    let inner = UnaryExpr::parse(token, vars)?;
                    let ty = inner.type_of_checked();

                    Ok(Expr {
                        internal: Box::new(ExprInternal::UnaryExpr(inner)),
                        ty,
                    })
                }
                _ => unreachable!("{:?}", token),
            },
            |lhs: Result<Expr>, op: Pair<Rule>, rhs: Result<Expr>| {
                let inner = BinaryExpr::new(span.clone(), lhs?, op, rhs?)?;
                let ty = inner.type_of_checked();

                Ok(Expr {
                    internal: Box::new(ExprInternal::BinaryExpr(inner)),
                    ty,
                })
            },
        )
    }
}

impl TypeOf for Expr {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
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
