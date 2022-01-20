use compiler::{err::Result, instance::Compiler};

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod compiler;

pub fn run_code<'s, S: Into<&'s str>>(source: S) -> Result<i64> {
    let compiler = Compiler::new();
    let result = compiler.module_from_source(source)?;
    Ok(result.exec()?)
}
