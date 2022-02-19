use super::{
    match_rule, Ast, BinaryExpr, Result, Rule, Term, Type, TypeOf, UnaryExpr, VisibleVars,
};
use lazy_static::lazy_static;
use pest::{
    iterators::Pair,
    prec_climber::{Operator, PrecClimber},
    Span,
};
use std::fmt::Display;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::Assoc::*;
        PrecClimber::new(vec![
            //
            Operator::new(Rule::add, Left) | Operator::new(Rule::sub, Left),

            //
            Operator::new(Rule::mul, Left) | Operator::new(Rule::div, Left),

            //
            Operator::new(Rule::eq, Left)
                | Operator::new(Rule::ne, Left)
                | Operator::new(Rule::gt, Left)
                | Operator::new(Rule::ge, Left)
                | Operator::new(Rule::lt, Left)
                | Operator::new(Rule::le, Left),

            //
            Operator::new(Rule::and, Left),

            //
            Operator::new(Rule::or, Left),
        ])
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprInternal<'i> {
    BinaryExpr(BinaryExpr<'i>),
    UnaryExpr(UnaryExpr<'i>),
    Term(Term<'i>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr<'i> {
    pub internal: Box<ExprInternal<'i>>,

    span: Span<'i>,
    ty: Option<Type>,
}

impl<'i> Ast<'i> for Expr<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::expr)?;

        PREC_CLIMBER.climb(
            token.into_inner(),
            |token: Pair<Rule>| match token.as_rule() {
                Rule::term => {
                    let inner = Term::parse(token)?;

                    Ok(Expr {
                        internal: Box::new(ExprInternal::Term(inner)),

                        span: span.clone(),
                        ty: None,
                    })
                }
                Rule::unary => {
                    let inner = UnaryExpr::parse(token)?;

                    Ok(Expr {
                        internal: Box::new(ExprInternal::UnaryExpr(inner)),

                        span: span.clone(),
                        ty: None,
                    })
                }
                _ => unreachable!("{:?}", token),
            },
            |lhs: Result<Expr>, op: Pair<Rule>, rhs: Result<Expr>| {
                let inner = BinaryExpr::new(span.clone(), lhs?, op, rhs?)?;

                Ok(Expr {
                    internal: Box::new(ExprInternal::BinaryExpr(inner)),

                    span: span.clone(),
                    ty: None,
                })
            },
        )
    }
}

impl<'i> TypeOf<'i> for Expr<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        let internal = match self.internal.as_mut() {
            ExprInternal::BinaryExpr(expr) => expr as &mut dyn TypeOf,
            ExprInternal::UnaryExpr(expr) => expr as _,
            ExprInternal::Term(term) => term as _,
        };

        internal.type_check(vars)?;
        self.ty = Some(internal.type_of());

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for Expr<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.internal.as_ref() {
            ExprInternal::BinaryExpr(v) => v as &dyn Display,
            ExprInternal::UnaryExpr(v) => v as _,
            ExprInternal::Term(v) => v as _,
        }
        .fmt(f)
    }
}
