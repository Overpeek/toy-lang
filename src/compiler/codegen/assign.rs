use super::{CodeGen, CodeGenResult};
use crate::{ast, compiler::module::Module};

//

impl CodeGen for ast::Assign {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        let value = self.expr.code_gen(module)?;

        module
            .function
            .borrow_mut()
            .as_mut()
            .expect("Assign outside of any function?")
            .vars
            .insert(self.name.value.clone(), value);

        Ok(value)
    }
}
