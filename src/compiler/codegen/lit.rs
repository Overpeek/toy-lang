use super::{CodeGen, CodeGenResult};
use crate::{ast, compiler::module::Module};

//

impl CodeGen for ast::Lit {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        match self {
            ast::Lit::F64(v) => v.code_gen(module),
            ast::Lit::I64(v) => v.code_gen(module),
            ast::Lit::Bool(v) => v.code_gen(module),
            ast::Lit::Unit(v) => v.code_gen(module),
        }
    }
}

impl CodeGen for f64 {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        Ok(Some(module.context.f64_type().const_float(*self).into()))
    }
}

impl CodeGen for i64 {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        Ok(Some(
            module
                .context
                .i64_type()
                .const_int(*self as u64, true)
                .into(),
        ))
    }
}

impl CodeGen for bool {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        Ok(Some(
            module
                .context
                .bool_type()
                .const_int(*self as u64, true)
                .into(),
        ))
    }
}

impl CodeGen for () {
    fn code_gen<'ctx>(&self, _: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        Ok(None)
    }
}
