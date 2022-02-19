use super::{
    codegen::CodeGen,
    err::{CompileResult, ExecuteError, ExecuteResult, Result},
    instance::Compiler,
    optimizer::OptLevel,
};
use crate::ast::{self, generic_mangle, Type, TypeOf};
use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module as LLModule,
    passes::{PassManager, PassManagerBuilder},
    values::{BasicValueEnum, FunctionValue},
    OptimizationLevel,
};
use std::{cell::RefCell, collections::HashMap, path::Path, rc::Rc};

//

pub(super) struct ScopeVars<'ctx> {
    pub proto: FunctionValue<'ctx>,
    pub vars: HashMap<String, Option<BasicValueEnum<'ctx>>>,
}

pub struct Module<'ctx> {
    pub(super) context: &'ctx Context,
    pub(super) module: LLModule<'ctx>,
    pub(super) builder: Builder<'ctx>,

    opt: OptLevel,
    lpm: PassManager<LLModule<'ctx>>,
    mpm: PassManager<LLModule<'ctx>>,
    fpm: PassManager<FunctionValue<'ctx>>,

    engine: ExecutionEngine<'ctx>,
    // main: Option<JitFunction<unsafe extern "C" fn()>>,
    ty: Type,
    pub label_id: u32,

    pub(super) functions: HashMap<String, FunctionValue<'ctx>>,
    pub(super) function: Rc<RefCell<Option<ScopeVars<'ctx>>>>, // current function and values
}

impl<'ctx> Module<'ctx> {
    pub fn new_from_path<P: AsRef<Path>>(
        compiler: &'ctx Compiler,
        path: P,
        opt: OptLevel,
    ) -> Result<Self> {
        let source = std::fs::read_to_string(path)?;
        Self::new_from_source(compiler, source.as_str(), opt)
    }

    pub fn new_from_source<'s, S: Into<&'s str>>(
        compiler: &'ctx Compiler,
        source: S,
        opt: OptLevel,
    ) -> Result<Self> {
        let module = ast::parse(source.into())?;
        Ok(Self::new_from_ast(compiler, &module, opt)?)
    }

    pub fn new_from_ast(
        compiler: &'ctx Compiler,
        ast_module: &ast::Module,
        opt: OptLevel,
    ) -> CompileResult<Self> {
        let context = &compiler.context;
        let module = context.create_module("repl");
        let builder = context.create_builder();

        let fpmb = PassManagerBuilder::create();
        fpmb.set_optimization_level(opt.into());
        fpmb.set_inliner_with_threshold(1024);

        let lpm = PassManager::create(&());
        let mpm = PassManager::create(&());
        let fpm = PassManager::create(&module);

        fpmb.populate_lto_pass_manager(&lpm, true, true);
        fpmb.populate_module_pass_manager(&mpm);
        fpmb.populate_function_pass_manager(&fpm);
        fpm.initialize();

        let engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let ty = ast_module
            .functions
            .get(&generic_mangle(&[], "__global"))
            .unwrap()
            .type_of();

        let mut module = Self {
            context,
            module,
            builder,

            opt,
            lpm,
            mpm,
            fpm,

            engine,
            // main: None,
            ty,
            label_id: 0,

            functions: HashMap::new(),
            function: Rc::new(RefCell::new(None)),
        };

        ast_module.code_gen(&mut module)?;
        module.finalize();

        /* // load the global function
        module.main = unsafe {
            module
                .engine
                .get_function::<unsafe extern "C" fn()>(&generic_mangle(&[], "__global"))
        }
        .ok(); */

        Ok(module)
    }

    pub fn get_function_0<T>(&self, name: &str) -> JitFunction<unsafe extern "C" fn() -> T> {
        unsafe {
            self.engine
                .get_function::<unsafe extern "C" fn() -> T>(&generic_mangle(&[], name))
        }
        .unwrap()
    }

    pub fn get_function_1<P1, T>(&self, name: &str) -> JitFunction<unsafe extern "C" fn(P1) -> T> {
        unsafe {
            self.engine
                .get_function::<unsafe extern "C" fn(P1) -> T>(&generic_mangle(&[], name))
        }
        .unwrap()
    }

    pub fn get_function_2<P1, P2, T>(
        &self,
        name: &str,
    ) -> JitFunction<unsafe extern "C" fn(P1, P2) -> T> {
        unsafe {
            self.engine
                .get_function::<unsafe extern "C" fn(P1, P2) -> T>(&generic_mangle(&[], name))
        }
        .unwrap()
    }

    pub fn exec<T: 'static>(&self) -> ExecuteResult<T> {
        if !self.ty.matches::<T>() {
            return Err(ExecuteError);
        }

        Ok(unsafe {
            self.engine
                .get_function::<unsafe extern "C" fn() -> T>(&generic_mangle(&[], "__global"))
                .unwrap()
                .call()
        })
    }

    fn finalize(&self) {
        for f in self.functions.values() {
            assert!(
                f.verify(true),
                "{}",
                self.module.print_to_string().to_string()
            );
        }

        log::debug!(
            "Non-Optimized LLVM IR: {}",
            self.module.print_to_string().to_string()
        );

        for f in self.functions.values() {
            if !matches!(self.opt, OptLevel::O0) {
                self.fpm.run_on(f);
            }
        }

        if !matches!(self.opt, OptLevel::O0) {
            self.mpm.run_on(&self.module);
            self.lpm.run_on(&self.module);
        }

        log::debug!(
            "Optimized LLVM IR: {}",
            self.module.print_to_string().to_string()
        );
    }
}
