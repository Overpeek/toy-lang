use super::{match_rule, Ast, Expr, Result, Rule, Type, VisibleVars};
use crate::ast::{Error, TypeOf};
use pest::{iterators::Pair, Span};
use std::{fmt::Display, hash::Hash};

//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// '+' <operand>
    Plus,

    /// '-' <operand>
    Neg,

    /// '!' <operand>
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr<'i> {
    pub operator: UnaryOp,
    pub operand: Box<Expr<'i>>,

    span: Span<'i>,
    ty: Option<Type>,
}

//

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

impl<'i> Ast<'i> for UnaryExpr<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::unary)?;

        let mut tokens = token.into_inner();
        let operator = tokens.next().unwrap();
        let operator = match operator.as_rule() {
            Rule::plus => UnaryOp::Plus,
            Rule::neg => UnaryOp::Neg,
            _ => unreachable!("{:?}", operator),
        };

        let operand = tokens.next().unwrap();
        let operand = Box::<Expr>::new(Ast::parse(operand)?);

        Ok(Self {
            operator,
            operand,

            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for UnaryExpr<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        self.operand.type_check(vars)?;

        let operator = self.operator;
        let operand = self.operand.type_of();

        let ty = match (operator, operand) {
            (UnaryOp::Plus, Type::F64) => Ok(Type::F64),
            (UnaryOp::Plus, Type::I64) => Ok(Type::I64),
            (UnaryOp::Neg, Type::F64) => Ok(Type::F64),
            (UnaryOp::Neg, Type::I64) => Ok(Type::I64),

            (UnaryOp::Not, Type::Bool) => Ok(Type::Bool),

            (_, Type::Unresolved) => Ok(Type::Unresolved),

            (op, rhs) => Err(Error::new_invalid_unary_op(
                Span::new("unreachable", 0, 0).unwrap(),
                op,
                rhs,
            )),
        }?;
        log::debug!("{operator} {operand} = {ty}");
        self.ty = Some(ty);

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for UnaryExpr<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.operand)
    }
}
