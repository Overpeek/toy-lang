use super::{CodeGen, CodeGenResult};
use crate::{
    ast,
    compiler::{err::CompileError, module::Module},
};

//

impl CodeGen for ast::UnaryExpr {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        let operand = self.operand.code_gen(module)?.unwrap();

        let value = match self.operator {
            ast::UnaryOp::Plus => operand,
            ast::UnaryOp::Neg => {
                if operand.is_float_value() {
                    let operand = operand.into_float_value();
                    module
                        .builder
                        .build_float_neg(operand, "UnaryExpr f neg")
                        .into()
                } else {
                    let operand = operand.into_int_value();
                    module
                        .builder
                        .build_int_neg(operand, "UnaryExpr i neg")
                        .into()
                }
            }
            ast::UnaryOp::Not => {
                if operand.is_float_value() {
                    return Err(CompileError::InvalidType);
                } else {
                    let operand = operand.into_int_value();
                    module.builder.build_not(operand, "UnaryExpr i not").into()
                }
            }
        };

        Ok(Some(value))
    }
}
