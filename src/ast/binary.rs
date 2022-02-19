use super::{Expr, Result, Rule, Type, TypeOf, VisibleVars};
use crate::ast::Error;
use pest::{iterators::Pair, Span};
use std::{fmt::Display, hash::Hash};

//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// <operand> '+' <operand>
    /// summed by
    Add,

    /// <operand> '-' <operand>
    /// subtracted by
    Sub,

    /// <operand> '*' <operand>
    /// multiplied by
    Mul,

    /// <operand> '/' <operand>
    /// divided by
    Div,

    /// <operand> '==' <operand>
    /// equal to
    Eq,

    /// <operand> '!=' <operand>
    /// not equal to
    Ne,

    /// <operand> '>' <operand>
    /// greater than
    Gt,

    /// <operand> '>=' <operand>
    /// greater than or equal to
    Ge,

    /// <operand> '<' <operand>
    /// less than
    Lt,

    /// <operand> '<=' <operand>
    /// less than or equal to
    Le,

    /// <operand> '||' <operand>
    /// or
    Or,

    /// <operand> '&&' <operand>
    /// and
    And,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sides<T> {
    pub lhs: T,
    pub rhs: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr<'i> {
    pub operator: BinaryOp,
    pub operands: Box<Sides<Expr<'i>>>,

    span: Span<'i>,
    ty: Option<Type>,
}

//

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),

            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Le => write!(f, "<="),

            BinaryOp::Or => write!(f, "||"),
            BinaryOp::And => write!(f, "&&"),
        }
    }
}

impl<'i> BinaryExpr<'i> {
    pub fn new(span: Span<'i>, lhs: Expr<'i>, op: Pair<'i, Rule>, rhs: Expr<'i>) -> Result<Self> {
        let operands = Box::new(Sides { lhs, rhs });
        let operator = match op.as_rule() {
            Rule::add => BinaryOp::Add,
            Rule::sub => BinaryOp::Sub,
            Rule::mul => BinaryOp::Mul,
            Rule::div => BinaryOp::Div,

            Rule::eq => BinaryOp::Eq,
            Rule::ne => BinaryOp::Ne,
            Rule::gt => BinaryOp::Gt,
            Rule::ge => BinaryOp::Ge,
            Rule::lt => BinaryOp::Lt,
            Rule::le => BinaryOp::Le,

            Rule::or => BinaryOp::Or,
            Rule::and => BinaryOp::And,

            _ => unreachable!("{:?}", op),
        };

        Ok(BinaryExpr {
            operator,
            operands,

            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for BinaryExpr<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        self.operands.lhs.type_check(vars)?;
        self.operands.rhs.type_check(vars)?;

        let op = self.operator;
        let lhs = self.operands.lhs.type_of();
        let rhs = self.operands.rhs.type_of();

        let ty = match (lhs, op, rhs) {
            // boolean ops
            (
                a,
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Or
                | BinaryOp::And,
                b,
            ) if a == b => Ok(Type::Bool),

            // arithmetic ops
            (Type::I64, _, Type::I64) => Ok(Type::I64),
            (Type::I64, _, Type::F64) => Ok(Type::F64),
            (Type::F64, _, Type::I64) => Ok(Type::F64),
            (Type::F64, _, Type::F64) => Ok(Type::F64),

            // generic ops
            (Type::Unresolved, _, Type::Unresolved) => Ok(Type::Unresolved),

            // invalid ops
            (lhs, op, rhs) => Err(Error::new_invalid_binary_op(
                Span::new("unreachable", 0, 0).unwrap(),
                lhs,
                op,
                rhs,
            )),
        }?;
        log::debug!("{lhs} {op} {rhs} = {ty}");
        self.ty = Some(ty);

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for BinaryExpr<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.operands.lhs, self.operator, self.operands.rhs
        )
    }
}
