use super::{Access, Ast, Branch, Call, Expr, Result, Rule, Type, TypeOf, VisibleVars};
use crate::ast::{match_rule, Lit};
use pest::{iterators::Pair, Span};
use std::fmt::{Debug, Display};

//

#[derive(Debug, Clone, PartialEq)]
pub enum TermInternal<'i> {
    Lit(Lit),
    Expr(Expr<'i>),
    Branch(Branch<'i>),
    Access(Access<'i>),
    Call(Call<'i>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Term<'i> {
    pub internal: Box<TermInternal<'i>>,

    span: Span<'i>,
    ty: Option<Type>,
}

//

impl<'i> Ast<'i> for Term<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::term)?;
        let mut tokens = token.into_inner();

        let token = tokens.next().unwrap();
        let internal = Box::new(match token.as_rule() {
            Rule::int => TermInternal::Lit(Lit::I64(token.as_str().parse().unwrap())),
            Rule::float => TermInternal::Lit(Lit::F64(token.as_str().parse().unwrap())),
            Rule::bool => TermInternal::Lit(Lit::Bool(token.as_str().parse().unwrap())),
            Rule::expr => TermInternal::Expr(Ast::parse(token)?),
            Rule::branch => TermInternal::Branch(Ast::parse(token)?),
            Rule::access => TermInternal::Access(Ast::parse(token)?),
            Rule::call => TermInternal::Call(Ast::parse(token)?),
            other => unreachable!("{:?}", other),
        });

        Ok(Term {
            internal,
            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for Term<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        let internal = match self.internal.as_mut() {
            TermInternal::Lit(v) => v as &mut dyn TypeOf<'i>,
            TermInternal::Expr(v) => v as _,
            TermInternal::Branch(v) => v as _,
            TermInternal::Access(v) => v as _,
            TermInternal::Call(v) => v as _,
        };

        internal.type_check(vars)?;
        self.ty = Some(internal.type_of());

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for Term<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.internal.as_ref() {
            TermInternal::Lit(v) => v as &dyn Display,
            TermInternal::Expr(v) => v as _,
            TermInternal::Branch(v) => v as _,
            TermInternal::Access(v) => v as _,
            TermInternal::Call(v) => v as _,
        }
        .fmt(f)
    }
}
