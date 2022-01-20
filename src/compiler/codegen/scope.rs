use super::{CodeGen, CodeGenResult};
use crate::{ast, compiler::module::Module};

//

impl CodeGen for ast::Scope {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        self.statements
            .iter()
            .map(|stmt| stmt.code_gen(module))
            .last()
            .unwrap_or(Ok(None))
    }
}
