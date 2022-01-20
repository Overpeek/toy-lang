use super::{CodeGen, CodeGenResult};
use crate::{ast, compiler::module::Module};

//

impl CodeGen for ast::Term {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        self.internal.code_gen(module)
    }
}

impl CodeGen for ast::TermInternal {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        match self {
            ast::TermInternal::Lit(lit) => lit.code_gen(module),
            ast::TermInternal::Expr(expr) => expr.code_gen(module),
            ast::TermInternal::Branch(branch) => branch.code_gen(module),
            ast::TermInternal::Access(access) => access.code_gen(module),
            ast::TermInternal::Call(call) => call.code_gen(module),
        }
    }
}
