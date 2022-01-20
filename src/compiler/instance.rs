use super::{err::Result, module::Module, optimizer::OptLevel};
use crate::ast;
use inkwell::context::Context;
use std::path::Path;

pub struct Compiler {
    pub(super) context: Context,
    pub opt: OptLevel,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_opt(mut self, opt: OptLevel) -> Self {
        self.opt = opt;
        self
    }

    pub fn module_from_path<P: AsRef<Path>>(&self, path: P) -> Result<Module> {
        Module::new_from_path(self, path, self.opt)
    }

    pub fn module_from_source<'s, S: Into<&'s str>>(&self, source: S) -> Result<Module> {
        Module::new_from_source(self, source, self.opt)
    }

    pub fn module_from_ast(&self, module: &ast::Module) -> Result<Module> {
        Ok(Module::new_from_ast(self, module, self.opt)?)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            context: Context::create(),
            opt: Default::default(),
        }
    }
}
