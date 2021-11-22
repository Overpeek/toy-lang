use std::fmt::Display;

use super::TypeOf;

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    F64(f64),
    I64(i64),
    Bool(bool),
    Unit(()),
}

impl TypeOf for Lit {
    fn type_of(&self) -> Type {
        match self {
            Lit::I64(_) => Type::I64,
            Lit::F64(_) => Type::F64,
            Lit::Bool(_) => Type::Bool,
            Lit::Unit(_) => Type::Unit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    /// `f64`
    F64,

    /// `i64`
    I64,

    /// `bool`
    Bool,

    /// `()`
    Unit,
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::F64(_) => todo!(),
            Lit::I64(_) => todo!(),
            Lit::Bool(_) => todo!(),
            Lit::Unit(_) => todo!(),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::F64 => write!(f, "f64"),
            Type::I64 => write!(f, "i64"),
            Type::Bool => write!(f, "bool"),
            Type::Unit => write!(f, "()"),
        }
    }
}
