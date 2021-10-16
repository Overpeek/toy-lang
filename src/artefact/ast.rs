use std::fmt::Debug;

use super::tokens::{LitFloat, LitInt, Operator};

pub trait Node: Debug {
    fn visit(&self) -> f64;
}

#[derive(Debug)]
pub struct BinaryOpNode {
    operator: Operator,
    lhs_node: Box<dyn Node>,
    rhs_node: Box<dyn Node>,
}

impl BinaryOpNode {
    pub fn new(operator: Operator, lhs_node: Box<dyn Node>, rhs_node: Box<dyn Node>) -> Self {
        Self {
            operator,
            lhs_node,
            rhs_node,
        }
    }
}

impl Node for BinaryOpNode {
    fn visit(&self) -> f64 {
        let lhs = self.lhs_node.visit();
        let rhs = self.rhs_node.visit();
        match self.operator {
            Operator::Add => lhs + rhs,
            Operator::Sub => lhs - rhs,
            Operator::Mul => lhs * rhs,
            Operator::Div => lhs / rhs,
        }
    }
}

#[derive(Debug)]
pub struct UnaryOpNode {
    operator: Operator,
    node: Box<dyn Node>,
}

impl UnaryOpNode {
    pub fn new(operator: Operator, node: Box<dyn Node>) -> Self {
        Self { operator, node }
    }
}

impl Node for UnaryOpNode {
    fn visit(&self) -> f64 {
        let lhs = self.node.visit();
        match self.operator {
            Operator::Add => lhs,
            Operator::Sub => -lhs,
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum NumberNode {
    LitInt(LitInt),
    LitFloat(LitFloat),
}

impl Node for NumberNode {
    fn visit(&self) -> f64 {
        match self {
            NumberNode::LitInt(i) => i.value as f64,
            NumberNode::LitFloat(f) => f.value,
        }
    }
}

#[derive(Debug)]
pub struct NoneNode {}

impl Node for NoneNode {
    fn visit(&self) -> f64 {
        unreachable!()
    }
}
