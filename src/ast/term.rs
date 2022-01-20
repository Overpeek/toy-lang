use crate::ast::Lit;

use super::{Access, Branch, Call, Expr, ParseAst, Result, Rule, Type, TypeOf, VisibleVars};
use pest::iterators::Pair;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum TermInternal {
    Lit(Lit),
    Expr(Expr),
    Branch(Branch),
    Access(Access),
    Call(Call),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub internal: Box<TermInternal>,

    ty: Option<Type>,
}

impl ParseAst for Term {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::term);

        let mut tokens = token.into_inner();
        let token = tokens.next().unwrap();

        let internal = Box::new(match token.as_rule() {
            Rule::int => TermInternal::Lit(Lit::I64(token.as_str().parse().unwrap())),
            Rule::float => TermInternal::Lit(Lit::F64(token.as_str().parse().unwrap())),
            Rule::bool => TermInternal::Lit(Lit::Bool(token.as_str().parse().unwrap())),
            Rule::expr => TermInternal::Expr(ParseAst::parse(token, vars)?),
            Rule::branch => TermInternal::Branch(ParseAst::parse(token, vars)?),
            Rule::access => TermInternal::Access(ParseAst::parse(token, vars)?),
            Rule::call => TermInternal::Call(ParseAst::parse(token, vars)?),
            other => unreachable!("{:?}", other),
        });
        let ty = match internal.as_ref() {
            TermInternal::Lit(v) => v as &dyn TypeOf,
            TermInternal::Expr(v) => v as _,
            TermInternal::Branch(v) => v as _,
            TermInternal::Access(v) => v as _,
            TermInternal::Call(v) => v as _,
        }
        .type_of_checked();

        Ok(Term { internal, ty })
    }
}

impl TypeOf for Term {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for Term {
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
