use crate::ast;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue};
use std::{
    fmt::{Debug, Display},
    io,
};

//

pub enum Error {
    ExecuteError(ExecuteError),
    CompileError(CompileError),
    IoError(io::Error),
    ParseError(ast::Error),
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ExecuteError(err) => err as &dyn Display,
            Error::CompileError(err) => err as _,
            Error::IoError(err) => err as _,
            Error::ParseError(err) => err as _,
        }
        .fmt(f)
    }
}

impl From<ExecuteError> for Error {
    fn from(val: ExecuteError) -> Self {
        Error::ExecuteError(val)
    }
}

impl From<CompileError> for Error {
    fn from(val: CompileError) -> Self {
        Error::CompileError(val)
    }
}

impl From<io::Error> for Error {
    fn from(val: io::Error) -> Self {
        Error::IoError(val)
    }
}

impl From<ast::Error> for Error {
    fn from(val: ast::Error) -> Self {
        Error::ParseError(val)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

//

pub struct ExecuteError;

impl Debug for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No main function")
    }
}

pub type ExecuteResult<T> = core::result::Result<T, ExecuteError>;

//

pub enum CompileError {
    InvalidType,
    VarNotFound,
    FuncNotFound,
}

impl Debug for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::InvalidType => write!(f, "Invalid type"),
            CompileError::VarNotFound => write!(f, "Var not found"),
            CompileError::FuncNotFound => write!(f, "Func not found"),
        }
    }
}

pub type CompileResult<T> = core::result::Result<T, CompileError>;

//

pub trait ExpectType<'ctx> {
    fn expect_float(self) -> CompileResult<FloatValue<'ctx>>;
    fn expect_int(self) -> CompileResult<IntValue<'ctx>>;
    fn expect_bool(self) -> CompileResult<IntValue<'ctx>>;
    fn expect_unit(self) -> CompileResult<()>;
}

impl<'ctx> ExpectType<'ctx> for Option<BasicValueEnum<'ctx>> {
    fn expect_float(self) -> CompileResult<FloatValue<'ctx>> {
        match self {
            Some(BasicValueEnum::FloatValue(val)) => Ok(val),
            _ => Err(CompileError::InvalidType),
        }
    }

    fn expect_int(self) -> CompileResult<IntValue<'ctx>> {
        match self {
            Some(BasicValueEnum::IntValue(val)) => Ok(val),
            _ => Err(CompileError::InvalidType),
        }
    }

    fn expect_bool(self) -> CompileResult<IntValue<'ctx>> {
        match self {
            Some(BasicValueEnum::IntValue(val)) => Ok(val),
            _ => Err(CompileError::InvalidType),
        }
    }

    fn expect_unit(self) -> CompileResult<()> {
        match self {
            None => Ok(()),
            _ => Err(CompileError::InvalidType),
        }
    }
}
