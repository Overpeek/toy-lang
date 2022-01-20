use super::TypeOf;
use std::fmt::Display;

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

    fn type_of_checked(&self) -> Option<Type> {
        Some(self.type_of())
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
            Lit::F64(v) => v as &dyn Display,
            Lit::I64(v) => v as _,
            Lit::Bool(v) => v as _,
            Lit::Unit(_) => &"()",
        }
        .fmt(f)
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
