use crate::artefact::{
    ast::{
        AccessNode, AssignNode, BinaryOpNode, BooleanNode, FnNode, IfElseNode, LitNode, Node,
        ScopeNode, UnaryOpNode,
    },
    tokens::{ErrorSpan, LitFloat, LitInt, LitStr, Operator, SourceType, Span},
};
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Display},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    OpNotImplementedForTypes(ErrorSpan, Operator, Node, Node),
    OpNotImplementedForType(ErrorSpan, Operator, Node),
    ExpectedBool(ErrorSpan, Node),
    NoSuchVar(ErrorSpan, String),
    CannotGhost(ErrorSpan, String),
    NoMainFn,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpNotImplementedForTypes(span, op, lhs, rhs) => write!(f, "Operator {} not implemented for rhs: {} lhs: {}\n{}", op, lhs, rhs, span),
            Self::OpNotImplementedForType(span, op, node) => write!(f, "Operator {} not implemented for: {}\n{}", op, node, span),
            Self::ExpectedBool(span, node) => write!(f, "Expected boolean, got: {}\n{}", node, span),
            Self::NoSuchVar(span, ident) => write!(f, "No variable {} was found\n{}", ident, span),
            Self::CannotGhost(span, ident) => write!(f, "Ghosting variable {}\n{}", ident, span),
            Self::NoMainFn => write!(f, "No main function defined"),
        }
    }
}

trait NodeVisit: Debug {
    fn visit(&self, mem: &mut Memory) -> Result<Cow<'_, Node>>;
}

pub type NodeVisitResult<'a> = Result<Cow<'a, Node>>;

impl NodeVisit for Node {
    fn visit(&self, mem: &mut Memory) -> NodeVisitResult<'_> {
        /* log::debug!(
            "visiting {{ {} }} = {{ {:?} }}",
            self,
            std::any::type_name::<Self>()
        ); */
        let result = match self {
            Self::BinaryOpNode(v) => v.visit(mem),
            Self::UnaryOpNode(v) => v.visit(mem),

            Self::LitNode(_) => Ok(Cow::Borrowed(self)),
            Self::NoneNode => Ok(Cow::Borrowed(self)),
            Self::BooleanNode(_) => Ok(Cow::Borrowed(self)),

            Self::IfElseNode(v) => v.visit(mem),
            Self::AssignNode(v) => v.visit(mem),
            Self::AccessNode(v) => v.visit(mem),
            Self::ScopeNode(v) => v.visit(mem),
            Self::FnNode(v) => v.visit(mem),
        };

        /* if let Ok(result) = &result {
            log::debug!("returned {{ {} }}", result);
        } */

        result
    }
}

impl NodeVisit for BinaryOpNode {
    fn visit(&self, mem: &mut Memory) -> NodeVisitResult<'_> {
        log::debug!(
            "visiting {{ {} }} = {{ {:?} }}",
            self,
            std::any::type_name::<Self>()
        );
        let lhs = self.nodes.0.visit(mem)?;
        let rhs = self.nodes.1.visit(mem)?;
        let result = match (self.operator, lhs.as_ref(), rhs.as_ref()) {
            (
                Operator::Add,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitFloat(LitFloat::new(
                lhs.value + rhs.value,
            ))))),
            (
                Operator::Add,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitInt(LitInt::new(
                lhs.value + rhs.value,
            ))))),
            (
                Operator::Add,
                Node::LitNode(LitNode::LitStr(lhs)),
                Node::LitNode(LitNode::LitStr(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitStr(LitStr::new(
                format!("{}{}", lhs.value, rhs.value),
            ))))),
            (
                Operator::Add,
                Node::LitNode(LitNode::LitChar(lhs)),
                Node::LitNode(LitNode::LitChar(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitStr(LitStr::new(
                format!("{}{}", lhs.value, rhs.value),
            ))))),
            (
                Operator::Add,
                Node::LitNode(LitNode::LitChar(lhs)),
                Node::LitNode(LitNode::LitStr(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitStr(LitStr::new(
                format!("{}{}", lhs.value, rhs.value),
            ))))),
            (
                Operator::Add,
                Node::LitNode(LitNode::LitStr(lhs)),
                Node::LitNode(LitNode::LitChar(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitStr(LitStr::new(
                format!("{}{}", lhs.value, rhs.value),
            ))))),
            (
                Operator::Sub,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitFloat(LitFloat::new(
                lhs.value - rhs.value,
            ))))),
            (
                Operator::Sub,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitInt(LitInt::new(
                lhs.value - rhs.value,
            ))))),
            (
                Operator::Mul,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitFloat(LitFloat::new(
                lhs.value * rhs.value,
            ))))),
            (
                Operator::Mul,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitInt(LitInt::new(
                lhs.value * rhs.value,
            ))))),
            (
                Operator::Mul,
                Node::LitNode(LitNode::LitStr(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitStr(LitStr::new(
                lhs.value.repeat(rhs.value as usize),
            ))))),
            (
                Operator::Div,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitFloat(LitFloat::new(
                lhs.value / rhs.value,
            ))))),
            (
                Operator::Div,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::LitNode(LitNode::LitInt(LitInt::new(
                lhs.value / rhs.value,
            ))))),

            (
                Operator::Eq,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value == rhs.value,
            )))),
            (
                Operator::Eq,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value == rhs.value,
            )))),
            (
                Operator::Ge,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value >= rhs.value,
            )))),
            (
                Operator::Ge,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value >= rhs.value,
            )))),
            (
                Operator::Gt,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value > rhs.value,
            )))),
            (
                Operator::Gt,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value > rhs.value,
            )))),
            (
                Operator::Le,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value <= rhs.value,
            )))),
            (
                Operator::Le,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value <= rhs.value,
            )))),
            (
                Operator::Lt,
                Node::LitNode(LitNode::LitFloat(lhs)),
                Node::LitNode(LitNode::LitFloat(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value < rhs.value,
            )))),
            (
                Operator::Lt,
                Node::LitNode(LitNode::LitInt(lhs)),
                Node::LitNode(LitNode::LitInt(rhs)),
            ) => Ok(Cow::Owned(Node::BooleanNode(BooleanNode::new(
                lhs.value < rhs.value,
            )))),

            (op, lhs, rhs) => Err(Error::OpNotImplementedForTypes(
                Span::new(0..0).make_error_span(&vec![], SourceType::Stdin),
                op,
                lhs.clone(),
                rhs.clone(),
            )),
        };

        if let Ok(result) = &result {
            log::debug!("returned {{ {} }}", result);
        }

        result
    }
}

impl NodeVisit for UnaryOpNode {
    fn visit(&self, mem: &mut Memory) -> NodeVisitResult<'_> {
        log::debug!(
            "visiting {{ {} }} = {{ {:?} }}",
            self,
            std::any::type_name::<Self>()
        );
        let node = self.node.visit(mem)?;
        let result = match (self.operator, node.as_ref()) {
            (Operator::Add, Node::LitNode(LitNode::LitFloat(node))) => Ok(Cow::Owned(
                Node::LitNode(LitNode::LitFloat(LitFloat::new(node.value))),
            )),
            (Operator::Add, Node::LitNode(LitNode::LitInt(node))) => Ok(Cow::Owned(Node::LitNode(
                LitNode::LitInt(LitInt::new(node.value)),
            ))),
            (Operator::Sub, Node::LitNode(LitNode::LitFloat(node))) => Ok(Cow::Owned(
                Node::LitNode(LitNode::LitFloat(LitFloat::new(-node.value))),
            )),
            (Operator::Sub, Node::LitNode(LitNode::LitInt(node))) => Ok(Cow::Owned(Node::LitNode(
                LitNode::LitInt(LitInt::new(-node.value)),
            ))),

            (op, node) => Err(Error::OpNotImplementedForType(
                Span::new(0..0).make_error_span(&vec![], SourceType::Stdin),
                op,
                node.clone(),
            )),
        };

        if let Ok(result) = &result {
            log::debug!("returned {{ {} }}", result);
        }

        result
    }
}

impl NodeVisit for IfElseNode {
    fn visit(&self, mem: &mut Memory) -> Result<Cow<'_, Node>> {
        log::debug!(
            "visiting {{ {} }} = {{ {:?} }}",
            self,
            std::any::type_name::<Self>()
        );
        let test = self.nodes.test.visit(mem)?;

        let result = match test.as_ref() {
            Node::BooleanNode(BooleanNode { value: true }) => self.nodes.on_true.visit(mem),
            Node::BooleanNode(BooleanNode { value: false }) => self.nodes.on_false.visit(mem),
            node => Err(Error::ExpectedBool(
                Span::new(0..0).make_error_span(&vec![], SourceType::Stdin),
                node.clone(),
            )),
        };

        if let Ok(result) = &result {
            log::debug!("returned {{ {} }}", result);
        }

        result
    }
}

impl NodeVisit for AssignNode {
    fn visit(&self, mem: &mut Memory) -> Result<Cow<'_, Node>> {
        log::debug!(
            "visiting {{ {} }} = {{ {:?} }}",
            self,
            std::any::type_name::<Self>()
        );
        let result = self.node.visit(mem)?.as_ref().to_owned();
        log::debug!("returned {{ {} }}", result);

        match mem.variables.insert(self.ident.clone(), result) {
            Some(_) => Err(Error::CannotGhost(
                Span::new(0..0).make_error_span(&vec![], SourceType::Stdin),
                self.ident.clone(),
            )),
            None => Ok(Cow::Owned(Node::NoneNode)),
        }
    }
}

impl NodeVisit for AccessNode {
    fn visit(&self, mem: &mut Memory) -> Result<Cow<'_, Node>> {
        log::debug!(
            "visiting {{ {} }} = {{ {:?} }}",
            self,
            std::any::type_name::<Self>()
        );
        let result = match mem.variables.get(&self.ident) {
            Some(node) => node.clone(),
            None => {
                return Err(Error::NoSuchVar(
                    Span::new(0..0).make_error_span(&vec![], SourceType::Stdin),
                    self.ident.clone(),
                ))
            }
        };

        log::debug!("returned {{ {} }}", result);

        Ok(Cow::Owned(result))
    }
}

impl NodeVisit for ScopeNode {
    fn visit(&self, mem: &mut Memory) -> Result<Cow<'_, Node>> {
        log::debug!(
            "visiting {{ {} }} = {{ {:?} }}",
            self,
            std::any::type_name::<Self>()
        );
        let mut last = Cow::Owned(Node::NoneNode);
        for line in self.lines.iter() {
            last = line.visit(mem)?;
        }

        log::debug!("returned {{ {} }}", last);

        Ok(last)
    }
}

impl NodeVisit for FnNode {
    fn visit(&self, mem: &mut Memory) -> Result<Cow<'_, Node>> {
        log::debug!(
            "visiting {{ {} }} = {{ {:?} }}",
            self,
            std::any::type_name::<Self>()
        );

        match mem.functions.insert(self.name.clone(), self.body.clone()) {
            Some(_) => Err(Error::CannotGhost(
                Span::new(0..0).make_error_span(&vec![], SourceType::Stdin),
                self.name.clone(),
            )),
            None => Ok(Cow::Owned(Node::NoneNode)),
        }
    }
}

pub struct Memory {
    pub variables: HashMap<String, Node>,
    pub functions: HashMap<String, ScopeNode>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

pub fn run_interpreter(ast: &Node) -> NodeVisitResult<'static> {
    let mut mem = Memory::new();
    run_interpreter_with(ast, &mut mem)
}

pub fn run_interpreter_with(ast: &Node, mem: &mut Memory) -> NodeVisitResult<'static> {
    ast.visit(mem)?;

    let main = mem.functions.get("main").cloned().ok_or(Error::NoMainFn)?;

    let result = main.visit(mem)?;
    log::debug!("got result {}", result);
    Ok(Cow::Owned(result.as_ref().clone()))
}
