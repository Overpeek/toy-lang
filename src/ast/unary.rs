use super::{Expr, ParseAst, Result, Rule, Type, VisibleVars};
use crate::ast::{Error, TypeOf};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// '+' <operand>
    Plus,

    /// '-' <operand>
    Neg,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Neg => write!(f, "-"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Box<Expr>,

    ty: Option<Type>,
}

impl ParseAst for UnaryExpr {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::unary);

        let span = token.as_span();

        let mut tokens = token.into_inner();
        let operator = tokens.next().unwrap();
        let operator = match operator.as_rule() {
            Rule::plus => UnaryOp::Plus,
            Rule::neg => UnaryOp::Neg,
            _ => unreachable!("{:?}", operator),
        };

        let operand = tokens.next().unwrap();
        let operand = Box::<Expr>::new(ParseAst::parse(operand, vars)?);

        let ty = match (operator, operand.type_of_checked()) {
            (UnaryOp::Plus, Some(Type::F64)) => Some(Type::F64),
            (UnaryOp::Plus, Some(Type::I64)) => Some(Type::I64),
            (UnaryOp::Neg, Some(Type::F64)) => Some(Type::F64),
            (UnaryOp::Neg, Some(Type::I64)) => Some(Type::I64),
            (op, Some(rhs)) => return Err(Error::new_invalid_unary_op(span, op, rhs)),
            (_, None) => None,
        };

        Ok(Self {
            operator,
            operand,

            ty,
        })
    }
}

impl TypeOf for UnaryExpr {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for UnaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.operand)
    }
}
