use super::{CodeGen, CodeGenResult};
use crate::{
    ast::{self, TypeOf},
    compiler::module::Module,
};

//

impl CodeGen for ast::Module {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        log::debug!("compiling module");

        // first compile all function prototypes

        for function in self.functions.values() {
            let name = function.internal.name.value.clone();
            let ty = function.type_of();

            let fn_ty = match ty {
                ast::Type::F64 => module.context.f64_type().fn_type(&[], false),
                ast::Type::I64 => module.context.i64_type().fn_type(&[], false),
                ast::Type::Bool => module.context.bool_type().fn_type(&[], false),
                ast::Type::Unit => module.context.void_type().fn_type(&[], false),
            };

            let proto = module.module.add_function(&name, fn_ty, None);

            module.functions.insert(name, proto);
        }

        // and then compile all function bodies

        for function in self.functions.values() {
            function.code_gen(module)?;
        }

        Ok(None)
    }
}
