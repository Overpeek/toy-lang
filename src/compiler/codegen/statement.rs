use super::{CodeGen, CodeGenResult};
use crate::{ast, compiler::module::Module};

//

impl<'i> CodeGen for ast::Statement<'i> {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        match self.internal.as_ref() {
            ast::StatementInternal::Expr(expr) => expr.code_gen(module),
            ast::StatementInternal::Assign(assign) => assign.code_gen(module),
        }
    }
}
