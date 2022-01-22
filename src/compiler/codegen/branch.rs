use super::{CodeGen, CodeGenResult};
use crate::{
    ast::{self, TypeOf},
    compiler::{
        err::{CompileError, ExpectType},
        module::Module,
    },
};
use inkwell::types::BasicTypeEnum;

// branches contain 3 blocks
//
//                               +---+
// +---+  on_true   +---+        | C |
// | E |------------| A |--------| O |
// | N |            +---+        | N |
// | T |                         | T |
// | R |  on_false  +---+        | I |
// | Y |------------| B |--------| N |
// +---+            +---+        | U |
//                               | E |
//                               +---+

impl<'i> CodeGen for ast::Branch<'i> {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        let cond = self.internal.test.code_gen(module)?.expect_bool()?;
        // let const_zero = module.context.f64_type().const_zero();
        let function = module.function.clone();
        let function = function.borrow();
        let function = function.as_ref().expect("Branch outside of any function?");

        // one block to enter if the condition is true
        let a = module
            .context
            .append_basic_block(function.proto, "Branch on_true");
        // one block to enter if the condition is false
        let b = module
            .context
            .append_basic_block(function.proto, "Branch on_false");
        // and one last block to enter after exiting from either one of the last blocks
        let r#continue = module
            .context
            .append_basic_block(function.proto, "Branch continue");

        module.builder.build_conditional_branch(cond, a, b);

        // on_true block
        module.builder.position_at_end(a);
        let result_a = self.internal.on_true.code_gen(module)?;
        module.builder.build_unconditional_branch(r#continue);

        // on_false block
        module.builder.position_at_end(b);
        let result_b = self.internal.on_false.code_gen(module)?;
        module.builder.build_unconditional_branch(r#continue);

        // continue block
        module.builder.position_at_end(r#continue);
        match (result_a, result_b) {
            (None, None) => Ok(None),
            (None, Some(_)) => Err(CompileError::InvalidType),
            (Some(_), None) => Err(CompileError::InvalidType),
            (Some(result_a), Some(result_b)) => {
                let ty: BasicTypeEnum = match self.type_of() {
                    ast::Type::F64 => module.context.f64_type().into(),
                    ast::Type::U64 => module.context.i64_type().into(),
                    ast::Type::I64 => module.context.i64_type().into(),
                    ast::Type::Bool => module.context.bool_type().into(),
                    ast::Type::Unit | ast::Type::Unresolved => unreachable!(),
                };

                let phi = module.builder.build_phi(ty, "Branch phi");
                phi.add_incoming(&[(&result_a, a), (&result_b, b)]);
                Ok(Some(phi.as_basic_value()))
            }
        }
    }
}
