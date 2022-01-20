use super::{CodeGen, CodeGenResult};
use crate::{ast, compiler::module::Module};
use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, FloatValue, IntValue},
    FloatPredicate, IntPredicate,
};

//

impl CodeGen for ast::BinaryExpr {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        let lhs = self.operands.lhs.code_gen(module)?.unwrap();
        let rhs = self.operands.rhs.code_gen(module)?.unwrap();

        let value = if lhs.is_float_value() || rhs.is_float_value() {
            let lhs = into_float_value(module, lhs);
            let rhs = into_float_value(module, rhs);
            let b = &module.builder;

            binary_float_op(b, self.operator, lhs, rhs)
        } else {
            let lhs = lhs.into_int_value();
            let rhs = rhs.into_int_value();
            let b = &module.builder;

            binary_int_op(b, self.operator, lhs, rhs)
        };

        Ok(Some(value))
    }
}

fn into_float_value<'ctx>(module: &Module<'ctx>, value: BasicValueEnum<'ctx>) -> FloatValue<'ctx> {
    if value.is_float_value() {
        value.into_float_value()
    } else {
        module.builder.build_signed_int_to_float(
            value.into_int_value(),
            module.context.f64_type(),
            "BinaryExpr lhs f cast",
        )
    }
}

fn binary_float_op<'ctx>(
    b: &Builder<'ctx>,
    op: ast::BinaryOp,
    lhs: FloatValue<'ctx>,
    rhs: FloatValue<'ctx>,
) -> BasicValueEnum<'ctx> {
    use ast::BinaryOp::*;
    use inkwell::FloatPredicate::*;

    let f_cmp = |pred: FloatPredicate, name: &str| -> BasicValueEnum {
        b.build_float_compare(pred, lhs, rhs, name).into()
    };

    match op {
        Add => b.build_float_add(lhs, rhs, "BinaryExpr f add").into(),
        Sub => b.build_float_sub(lhs, rhs, "BinaryExpr f sub").into(),
        Mul => b.build_float_mul(lhs, rhs, "BinaryExpr f mul").into(),
        Div => b.build_float_div(lhs, rhs, "BinaryExpr f div").into(),

        Eq => f_cmp(OEQ, "BinaryExpr f eq"),
        Ne => f_cmp(ONE, "BinaryExpr f neq"),
        Gt => f_cmp(OGT, "BinaryExpr f gt"),
        Ge => f_cmp(OGE, "BinaryExpr f ge"),
        Lt => f_cmp(OLT, "BinaryExpr f lt"),
        Le => f_cmp(OLE, "BinaryExpr f le"),
    }
}

fn binary_int_op<'ctx>(
    b: &Builder<'ctx>,
    op: ast::BinaryOp,
    lhs: IntValue<'ctx>,
    rhs: IntValue<'ctx>,
) -> BasicValueEnum<'ctx> {
    use ast::BinaryOp::*;
    use inkwell::IntPredicate::*;

    let i_cmp = |pred: IntPredicate, name: &str| -> BasicValueEnum {
        b.build_int_compare(pred, lhs, rhs, name).into()
    };

    match op {
        Add => b.build_int_add(lhs, rhs, "BinaryExpr i add").into(),
        Sub => b.build_int_sub(lhs, rhs, "BinaryExpr i sub").into(),
        Mul => b.build_int_mul(lhs, rhs, "BinaryExpr i mul").into(),
        Div => b.build_int_signed_div(lhs, rhs, "BinaryExpr i div").into(),

        Eq => i_cmp(EQ, "BinaryExpr i eq"),
        Ne => i_cmp(NE, "BinaryExpr i ne"),
        Gt => i_cmp(SGT, "BinaryExpr i gt"),
        Ge => i_cmp(SGE, "BinaryExpr i ge"),
        Lt => i_cmp(SLT, "BinaryExpr i lt"),
        Le => i_cmp(SLE, "BinaryExpr i le"),
    }
}
