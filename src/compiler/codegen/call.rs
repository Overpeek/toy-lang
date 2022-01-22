use super::{CodeGen, CodeGenResult};
use crate::{
    ast::{self, generic_mangle, TypeOf},
    compiler::{err::CompileError, module::Module},
};

//

impl<'i> CodeGen for ast::Call<'i> {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        let func = match module.functions.get(self.name.value.as_str()) {
            Some(&val) => val,
            None => {
                let as_generic = generic_mangle(
                    &self
                        .args
                        .iter()
                        .map(|arg| arg.type_of())
                        .collect::<Box<_>>(),
                    self.name.value.as_str(),
                );

                match module.functions.get(&as_generic) {
                    Some(&val) => val,
                    None => return Err(CompileError::FuncNotFound),
                }
            }
        };

        let args: Vec<_> = self
            .args
            .iter()
            .map(|arg| Ok(arg.code_gen(module)?.unwrap()))
            .collect::<Result<_, _>>()?;

        let ret = module
            .builder
            .build_call(func, &args[..], "function call")
            .try_as_basic_value()
            .left()
            .unwrap();

        Ok(Some(ret))
    }
}
