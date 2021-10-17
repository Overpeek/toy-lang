use super::tokens::{Lit, LitChar, LitFloat, LitInt, LitStr, Operator};
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    BinaryOpNode(BinaryOpNode),
    UnaryOpNode(UnaryOpNode),
    LitNode(LitNode),
    NoneNode,
    BooleanNode(BooleanNode),
    IfElseNode(IfElseNode),
    AssignNode(AssignNode),
    AccessNode(AccessNode),
    ScopeNode(ScopeNode),
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryOpNode(v) => Display::fmt(v, f),
            Self::UnaryOpNode(v) => Display::fmt(v, f),
            Self::LitNode(v) => Display::fmt(v, f),
            Self::NoneNode => write!(f, "()"),
            Self::BooleanNode(v) => Display::fmt(v, f),
            Self::IfElseNode(v) => Display::fmt(v, f),
            Self::AssignNode(v) => Display::fmt(v, f),
            Self::AccessNode(v) => Display::fmt(v, f),
            Self::ScopeNode(v) => Display::fmt(v, f),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOpNode {
    pub operator: Operator,
    pub nodes: Box<(Node, Node)>,
}

impl BinaryOpNode {
    pub fn new(operator: Operator, lhs_node: Node, rhs_node: Node) -> Self {
        Self {
            operator,
            nodes: Box::new((lhs_node, rhs_node)),
        }
    }
}

impl Display for BinaryOpNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.nodes.0, self.operator, self.nodes.1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOpNode {
    pub operator: Operator,
    pub node: Box<Node>,
}

impl UnaryOpNode {
    pub fn new(operator: Operator, node: Node) -> Self {
        Self {
            operator,
            node: Box::new(node),
        }
    }
}

impl Display for UnaryOpNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.node)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LitNode {
    LitInt(LitInt),
    LitFloat(LitFloat),
    LitStr(LitStr),
    LitChar(LitChar),
}

impl From<Lit> for LitNode {
    fn from(lit: Lit) -> Self {
        match lit {
            Lit::LitInt(v) => LitNode::LitInt(v),
            Lit::LitFloat(v) => LitNode::LitFloat(v),
            Lit::LitStr(v) => LitNode::LitStr(v),
            Lit::LitChar(v) => LitNode::LitChar(v),
        }
    }
}

impl Display for LitNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LitInt(v) => Display::fmt(v, f),
            Self::LitFloat(v) => Display::fmt(v, f),
            Self::LitStr(v) => Display::fmt(v, f),
            Self::LitChar(v) => Display::fmt(v, f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BooleanNode {
    pub value: bool,
}

impl BooleanNode {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl Display for BooleanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfElseNodeInternal {
    pub test: Node,
    pub on_true: Node,
    pub on_false: Node,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfElseNode {
    pub nodes: Box<IfElseNodeInternal>,
}

impl IfElseNode {
    pub fn new(test: Node, on_true: Node, on_false: Node) -> Self {
        Self {
            nodes: Box::new(IfElseNodeInternal {
                test,
                on_true,
                on_false,
            }),
        }
    }
}

impl Display for IfElseNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "if {} {{ {} }} else {{ {} }}",
            self.nodes.test, self.nodes.on_true, self.nodes.on_false
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignNode {
    pub ident: String,
    pub node: Box<Node>,
}

impl AssignNode {
    pub fn new(ident: String, node: Node) -> Self {
        Self {
            ident,
            node: Box::new(node),
        }
    }
}

impl Display for AssignNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {}", self.ident, self.node)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccessNode {
    pub ident: String,
}

impl AccessNode {
    pub fn new(ident: String) -> Self {
        Self { ident }
    }
}

impl Display for AccessNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeNode {
    pub lines: Vec<Node>,
}

impl ScopeNode {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn push_line(&mut self, node: Node) {
        self.lines.push(node)
    }
}

impl Display for ScopeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(for line in self.lines.iter() {
            writeln!(f, "{};", line)?;
        })
    }
}
