use inkwell::types::BasicTypeEnum;

use super::{CodeGen, CodeGenResult};
use crate::{
    ast::{self, Type, TypeOf},
    compiler::module::Module,
};

//

impl<'i> CodeGen for ast::Module<'i> {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx> {
        log::debug!("compiling module");

        // first compile all function prototypes

        for function in self.functions.values() {
            let name = function.internal.name.value.clone();
            let ty = function.type_of();

            let params: Vec<BasicTypeEnum> = function
                .internal
                .params
                .iter()
                .map(|param| match param.ty {
                    Type::F64 => module.context.f64_type().into(),
                    Type::I64 => module.context.i64_type().into(),
                    Type::U64 => module.context.i64_type().into(),
                    Type::Bool => module.context.bool_type().into(),
                    Type::Unit | Type::Unresolved => unreachable!(),
                })
                .collect();

            let fn_ty = match ty {
                Type::F64 => module.context.f64_type().fn_type(&params[..], false),
                Type::U64 => module.context.i64_type().fn_type(&params[..], false),
                Type::I64 => module.context.i64_type().fn_type(&params[..], false),
                Type::Bool => module.context.bool_type().fn_type(&params[..], false),
                Type::Unit => module.context.void_type().fn_type(&params[..], false),
                Type::Unresolved => unreachable!(),
            };

            log::debug!("compiling proto: '{}' -> {:?}", name, ty);
            let proto = module.module.add_function(&name, fn_ty, None);
            for (param, param_name) in proto.get_param_iter().zip(function.internal.params.iter()) {
                match param {
                    inkwell::values::BasicValueEnum::ArrayValue(v) => {
                        v.set_name(param_name.ident.value.as_str())
                    }
                    inkwell::values::BasicValueEnum::IntValue(v) => {
                        v.set_name(param_name.ident.value.as_str())
                    }
                    inkwell::values::BasicValueEnum::FloatValue(v) => {
                        v.set_name(param_name.ident.value.as_str())
                    }
                    inkwell::values::BasicValueEnum::PointerValue(v) => {
                        v.set_name(param_name.ident.value.as_str())
                    }
                    inkwell::values::BasicValueEnum::StructValue(v) => {
                        v.set_name(param_name.ident.value.as_str())
                    }
                    inkwell::values::BasicValueEnum::VectorValue(v) => {
                        v.set_name(param_name.ident.value.as_str())
                    }
                }
            }
            module.functions.insert(name, proto);
        }

        // and then compile all function bodies

        for function in self.functions.values() {
            function.code_gen(module)?;
        }

        Ok(None)
    }
}
