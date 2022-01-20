use super::{CodeGen, CodeGenResult};
use crate::{ast, compiler::module::Module};

//

impl CodeGen for ast::Expr {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        self.internal.code_gen(module)
    }
}

impl CodeGen for ast::ExprInternal {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        match self {
            ast::ExprInternal::BinaryExpr(expr) => expr.code_gen(module),
            ast::ExprInternal::UnaryExpr(expr) => expr.code_gen(module),
            ast::ExprInternal::Term(term) => term.code_gen(module),
        }
    }
}
