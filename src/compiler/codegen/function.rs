use super::{CodeGen, CodeGenResult};
use crate::{
    ast::{self, TypeOf},
    compiler::module::{Module, ScopeVars},
};
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

//

impl<'i> CodeGen for ast::Function<'i> {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        let proto = match module.functions.get(self.internal.name.value.as_str()) {
            Some(&proto) => proto,
            None => return Ok(None),
        };

        log::debug!(
            "compiling fn: '{}' -> {}",
            self.internal.name,
            self.type_of()
        );

        // block
        let entry = module.context.append_basic_block(proto, "entry");
        module.builder.position_at_end(entry);

        // setup scope vars
        let mut vars: HashMap<String, Option<BasicValueEnum>> = HashMap::new();
        for (param, param_name) in proto.get_param_iter().zip(self.internal.params.iter()) {
            vars.insert(param_name.ident.value.clone(), Some(param));
        }

        // scope
        *module.function.borrow_mut() = Some(ScopeVars { proto, vars });
        let value = self.internal.scope.code_gen(module)?.unwrap();

        // return
        module.builder.build_return(Some(&value));
        *module.function.borrow_mut() = None;

        Ok(None)
    }
}
