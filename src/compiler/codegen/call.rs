use super::{CodeGen, CodeGenResult};
use crate::{
    ast,
    compiler::{err::CompileError, module::Module},
};

//

impl CodeGen for ast::Call {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        let func = match module.functions.get(self.name.value.as_str()) {
            Some(&val) => val,
            None => return Err(CompileError::FuncNotFound),
        };

        let ret = module
            .builder
            .build_call(func, &[], "function call")
            .try_as_basic_value()
            .left()
            .unwrap();

        Ok(Some(ret))
    }
}
