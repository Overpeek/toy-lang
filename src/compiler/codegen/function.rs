use super::{CodeGen, CodeGenResult};
use crate::{
    ast,
    compiler::module::{Module, ScopeVars},
};
use std::collections::HashMap;

//

impl CodeGen for ast::Function {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        log::debug!("compiling fn: '{}'", self.internal.name);

        let proto = *module
            .functions
            .get(self.internal.name.value.as_str())
            .unwrap();

        // block
        let entry = module.context.append_basic_block(proto, "entry");
        module.builder.position_at_end(entry);

        // scope
        *module.function.borrow_mut() = Some(ScopeVars {
            proto,
            vars: HashMap::new(),
        });
        let value = self.internal.scope.code_gen(module)?.unwrap();

        // return
        module.builder.build_return(Some(&value));
        *module.function.borrow_mut() = None;

        Ok(None)
    }
}
