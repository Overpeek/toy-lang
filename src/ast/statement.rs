use super::{
    match_rule, Assign, Ast, Expr, GenericSolver, Result, Rule, Type, TypeOf, VisibleVars,
};
use pest::{iterators::Pair, Span};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementInternal<'i> {
    Expr(Expr<'i>),
    Assign(Assign<'i>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement<'i> {
    pub internal: Box<StatementInternal<'i>>,

    span: Span<'i>,
    ty: Option<Type>,
}

impl<'i> Ast<'i> for Statement<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::statement)?;
        let mut tokens = token.into_inner();

        let token = tokens.next().unwrap();
        let internal = Box::new(match token.as_rule() {
            Rule::expr => StatementInternal::Expr(Ast::parse(token)?),
            Rule::assign => StatementInternal::Assign(Ast::parse(token)?),
            _ => unreachable!("{:?}", token),
        });
        assert_eq!(tokens.next(), None);

        Ok(Statement {
            internal,
            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for Statement<'i> {
    fn type_check_impl(
        &mut self,
        vars: &mut VisibleVars,
        solver: &mut GenericSolver<'i>,
    ) -> Result<()> {
        let internal = match self.internal.as_mut() {
            StatementInternal::Expr(expr) => expr as &mut dyn TypeOf,
            StatementInternal::Assign(assign) => assign as _,
        };

        internal.type_check(vars, solver)?;
        self.ty = Some(internal.type_of());

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for Statement<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.internal.as_ref() {
            StatementInternal::Expr(v) => v as &dyn Display,
            StatementInternal::Assign(v) => v as _,
        }
        .fmt(f)
    }
}
