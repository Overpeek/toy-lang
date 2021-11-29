use super::{Expr, Result, Rule, Type, TypeOf};
use crate::ast::Error;
use pest::{iterators::Pair, Span};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// <operand> '+' <operand>
    Add,

    /// <operand> '-' <operand>
    Sub,

    /// <operand> '*' <operand>
    Mul,

    /// <operand> '/' <operand>
    Div,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sides<T> {
    pub lhs: T,
    pub rhs: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub operator: BinaryOp,
    pub operands: Box<Sides<Expr>>,

    ty: Option<Type>,
}

impl BinaryExpr {
    pub fn new(span: Span, lhs: Expr, op: Pair<Rule>, rhs: Expr) -> Result<Self> {
        let (ty_lhs, ty_rhs) = (lhs.type_of_checked(), rhs.type_of_checked());
        let operands = Box::new(Sides { lhs, rhs });
        let operator = match op.as_rule() {
            Rule::add => BinaryOp::Add,
            Rule::sub => BinaryOp::Sub,
            Rule::mul => BinaryOp::Mul,
            Rule::div => BinaryOp::Div,
            _ => unreachable!("{:?}", op),
        };

        let ty = match (ty_lhs, operator, ty_rhs) {
            (Some(Type::I64), _, Some(Type::I64)) => Some(Type::I64),
            (Some(Type::I64), _, Some(Type::F64)) => Some(Type::F64),
            (Some(Type::F64), _, Some(Type::I64)) => Some(Type::F64),
            (Some(Type::F64), _, Some(Type::F64)) => Some(Type::F64),
            (Some(lhs), op, Some(rhs)) => {
                return Err(Error::new_invalid_binary_op(span, lhs, op, rhs))
            }
            (None, _, _) => None,
            (_, _, None) => None,
        };

        Ok(BinaryExpr {
            operator,
            operands,

            ty,
        })
    }
}

impl TypeOf for BinaryExpr {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for BinaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.operands.lhs, self.operator, self.operands.rhs
        )
    }
}
