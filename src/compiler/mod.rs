use inkwell::OptimizationLevel;

use crate::artefact::ast::Node;

/// TODO: JIT compile with llvm

type Func = unsafe extern "C" fn(u64, u64, u64) -> u64;

pub fn run_llvm_jit(_node: Node) -> f32 {
	


    todo!();
    let context = inkwell::context::Context::create();
    let module = context.create_module("module");
    let builder = context.create_builder();
    let exec_engine = module
        .create_jit_execution_engine(OptimizationLevel::Aggressive)
        .unwrap();

    let i64_t = context.i64_type();
    let func_t = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i64_t.into()], false);
    let func = module.add_function("func", func_t, None);
    let basic_block = context.append_basic_block(func, "entry");

    builder.position_at_end(basic_block);

    let x = func.get_nth_param(0).unwrap().into_int_value();
    let y = func.get_nth_param(1).unwrap().into_int_value();
    let z = func.get_nth_param(2).unwrap().into_int_value();

    let sum = builder.build_int_add(x, y, "sum");
    let sum = builder.build_int_add(sum, z, "sum");

    builder.build_return(Some(&sum));

    let func = unsafe { exec_engine.get_function::<Func>("func") }.unwrap();
    let result = unsafe { func.call(3, 4, 5) };

    println!("LLVM compiled mess returned with result: {}", result);
}
