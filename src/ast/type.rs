use super::{Ast, Generic, GenericSolver, Result, Rule, TypeOf, VisibleVars};
use crate::ast::match_rule;
use pest::{iterators::Pair, Span};
use std::{
    fmt::{Debug, Display, Formatter},
    hash::Hash,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    F64(f64),
    I64(i64),
    Bool(bool),
    Unit(()),
}

impl<'i> TypeOf<'i> for Lit {
    fn type_check_impl(&mut self, _: &mut VisibleVars, _: &mut GenericSolver<'i>) -> Result<()> {
        Ok(())
    }

    fn type_of(&self) -> Type {
        match self {
            Lit::I64(_) => Type::I64,
            Lit::F64(_) => Type::F64,
            Lit::Bool(_) => Type::Bool,
            Lit::Unit(_) => Type::Unit,
        }
    }

    fn type_of_impl(&self) -> Option<Type> {
        Some(self.type_of())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    /// `f64`
    F64,

    /// `i64`
    I64,

    /// `u64`
    U64,

    /// `bool`
    Bool,

    /// `()`
    Unit,

    Unresolved,
    /* /// `unresolved type`
    Generic(u64), */
}

impl Generic for Type {
    fn eval(self, _: &mut GenericSolver) -> Result<Type> {
        Ok(self)
    }
}

impl<'i> Ast<'i> for Type {
    fn span(&self) -> Span<'i> {
        Span::new("unreachable", 0, 11).unwrap()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::ty)?;
        let mut tokens = token.into_inner();

        let token = tokens.next().unwrap();
        Ok(match token.as_rule() {
            Rule::unit_ty => Self::Unit,
            Rule::bool_ty => Self::Bool,
            Rule::u_ty => Self::U64,
            Rule::i_ty => Self::I64,
            Rule::f_ty => Self::F64,
            // Rule::gen => Self::Unresolved,
            _ => unreachable!(),
        })
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F64(v) => v as &dyn Display,
            Self::I64(v) => v as _,
            Self::Bool(v) => v as _,
            Self::Unit(_) => &"()",
        }
        .fmt(f)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F64 => write!(f, "f64"),
            Self::U64 => write!(f, "u64"),
            Self::I64 => write!(f, "i64"),
            Self::Bool => write!(f, "bool"),
            Self::Unit => write!(f, "()"),
            Self::Unresolved => write!(f, "<?>"),
        }
    }
}
