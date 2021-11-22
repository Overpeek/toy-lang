use crate::ast::{Error, TypeOf};

use super::{Expr, ParseAst, Result, Rule, Type};
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
    ty: Type,
}

impl ParseAst for UnaryExpr {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::unary);

        let mut tokens = token.into_inner();
        let operator = tokens.next().unwrap();
        let operator = match operator.as_rule() {
            Rule::plus => UnaryOp::Plus,
            Rule::neg => UnaryOp::Neg,
            _ => unreachable!("{:?}", operator),
        };

        let operand = tokens.next().unwrap();
        let operand = Box::<Expr>::new(ParseAst::parse(operand)?);

        let ty = match (operator, operand.type_of()) {
            (UnaryOp::Plus, Type::I64) => Type::I64,
            (UnaryOp::Neg, Type::F64) => Type::F64,
            (op, rhs) => return Err(Error::InvalidUnaryOp(op, rhs)),
        };

        Ok(Self {
            operator,
            operand,
            ty,
        })
    }
}

impl TypeOf for UnaryExpr {
    fn type_of(&self) -> Type {
        self.ty
    }
}

impl Display for UnaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.operand)
    }
}
