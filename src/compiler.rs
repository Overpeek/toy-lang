use crate::ast::{self, Ident};
use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::{Linkage, Module},
    passes::{PassManager, PassManagerBuilder},
    values::{ArrayValue, BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue},
    AddressSpace, IntPredicate, OptimizationLevel,
};
use std::collections::HashMap;

pub struct Compiler {
    context: Context,
}

pub struct CompilerModule<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    lpm: PassManager<Module<'ctx>>,
    mpm: PassManager<Module<'ctx>>,
    fpm: PassManager<FunctionValue<'ctx>>,

    engine: ExecutionEngine<'ctx>,
    main: Option<JitFunction<unsafe extern "C" fn() -> f64>>,

    functions: HashMap<String, FunctionValue<'ctx>>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            context: Context::create(),
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn module(&mut self, ast: &ast::Module) -> CompilerModule<'_> {
        let context = &self.context;
        let module = context.create_module("repl");
        let builder = context.create_builder();

        let fpmb = PassManagerBuilder::create();
        fpmb.set_optimization_level(OptimizationLevel::Aggressive);
        fpmb.set_inliner_with_threshold(100);

        let lpm = PassManager::create(&());
        let mpm = PassManager::create(&());
        let fpm = PassManager::create(&module);

        fpmb.populate_lto_pass_manager(&lpm, true, true);
        fpmb.populate_module_pass_manager(&mpm);
        fpmb.populate_function_pass_manager(&fpm);
        fpm.initialize();

        let engine = module
            .create_jit_execution_engine(OptimizationLevel::Aggressive)
            .unwrap();

        let mut module = CompilerModule {
            context,
            module,
            builder,

            lpm,
            mpm,
            fpm,

            engine,
            main: None,

            functions: HashMap::new(),
        };

        module.compile_mod(ast);
        module.dump();

        // load the main function
        module.main = unsafe {
            module
                .engine
                .get_function::<unsafe extern "C" fn() -> f64>("main")
        }
        .ok();

        module
    }
}

impl<'ctx> CompilerModule<'ctx> {
    pub fn exec(&self) -> f64 {
        unsafe { self.main.as_ref().expect("No main function").call() }
    }

    pub fn dump(&self) {
        self.mpm.run_on(&self.module);
        self.lpm.run_on(&self.module);
        log::debug!("LLVM IR: {}", self.module.print_to_string().to_string());
    }

    fn compile_mod(&mut self, module: &ast::Module) {
        log::debug!("compiling module");

        for function in module.functions.values() {
            self.compile_fn(function);
        }
    }

    fn compile_fn(&mut self, function: &ast::Function) -> FunctionValue<'ctx> {
        log::debug!("compiling fn: '{}'", function.internal.name);

        let name = &function.internal.name;
        let scope = &function.internal.scope;

        // function prototype and block
        let proto = self.compile_fn_proto(name);
        let entry = self.context.append_basic_block(proto, "entry");
        self.builder.position_at_end(entry);

        let value = self.compile_scope(scope);

        // function return
        self.builder.build_return(Some(&value));

        // verification
        assert!(proto.verify(true));
        self.fpm.run_on(&proto);
        self.functions.insert(name.value.clone(), proto);
        proto
    }

    fn compile_fn_proto(&self, name: &Ident) -> FunctionValue<'ctx> {
        let fn_type = self.context.f64_type().fn_type(&[], false);
        self.module.add_function(name.value.as_str(), fn_type, None)
    }

    fn compile_scope(&self, scope: &ast::Scope) -> FloatValue {
        let mut last = self.context.f64_type().const_float(0.0);

        for statement in scope.statements.iter() {
            last = self.compile_statement(statement);
        }

        last
    }

    fn compile_statement(&self, statement: &ast::Statement) -> FloatValue {
        todo!()
        // self.compile_expr(statement)
    }

    fn compile_expr(&self, expr: &ast::Expr) -> FloatValue {
        let mut lhs = self.compile_term(expr.internal.);
        for (op, rhs) in expr.1.iter() {
            let rhs = self.compile_term(rhs);
            match op {
                ast::Either::A(_) => lhs = self.builder.build_float_add(lhs, rhs, "add"),
                ast::Either::B(_) => lhs = self.builder.build_float_sub(lhs, rhs, "sub"),
            }
        }
        lhs
    }

    fn compile_term(&self, term: &ast::Term) -> FloatValue {
        let mut lhs = self.compile_factor(&term.0);
        for (op, rhs) in term.1.iter() {
            let rhs = self.compile_factor(rhs);
            match op {
                ast::Either::A(_) => lhs = self.builder.build_float_mul(lhs, rhs, "mul"),
                ast::Either::B(_) => lhs = self.builder.build_float_div(lhs, rhs, "div"),
            }
        }
        lhs
    }

    fn compile_factor(&self, factor: &ast::Factor) -> FloatValue {
        match factor {
            ast::Factor::F64(f) => self.compile_float(f),
            ast::Factor::I64(i) => self.compile_int(i),
            ast::Factor::Expr(expr) => self.compile_expr(expr),
            ast::Factor::Call(call) => self.compile_call(call),
            ast::Factor::Sign(sign) => {
                let rhs = self.compile_factor(&sign.1);
                match &sign.0 {
                    ast::Either::A(_) => rhs,
                    ast::Either::B(_) => self.builder.build_float_neg(rhs, "neg"),
                }
            }
            ast::Factor::Access(access) => todo!(),
        }
    }

    fn compile_float(&self, float: &f64) -> FloatValue {
        self.context.f64_type().const_float(*float)
    }

    fn compile_int(&self, float: &i64) -> FloatValue {
        self.context.f64_type().const_float(*float as f64) // TODO:
    }

    fn compile_call(&self, call: &ast::Call) -> FloatValue {
        let func = *self.functions.get(&call.0).expect("Unknown function");
        let ret = self
            .builder
            .build_call(func, &[], "function call")
            .try_as_basic_value()
            .left()
            .unwrap();

        match ret {
            BasicValueEnum::FloatValue(float) => float,
            _ => unimplemented!(),
        }
    }
}
