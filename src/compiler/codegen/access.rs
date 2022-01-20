use super::{CodeGen, CodeGenResult};
use crate::{
    ast,
    compiler::{err::CompileError, module::Module},
};

//

impl CodeGen for ast::Access {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        let function = module.function.borrow();
        let function = function.as_ref().expect("Access outside of any function?");

        match function.vars.get(self.name.value.as_str()) {
            Some(&val) => Ok(val),
            None => Err(CompileError::VarNotFound),
        }
    }
}
