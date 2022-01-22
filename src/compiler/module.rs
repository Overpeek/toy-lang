use super::{
    codegen::CodeGen,
    err::{CompileResult, ExecuteError, ExecuteResult, Result},
    instance::Compiler,
    optimizer::OptLevel,
};
use crate::ast::{self, generic_mangle};
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
    main: Option<JitFunction<unsafe extern "C" fn() -> i64>>,

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

        let mut module = Self {
            context,
            module,
            builder,

            opt,
            lpm,
            mpm,
            fpm,

            engine,
            main: None,

            functions: HashMap::new(),
            function: Rc::new(RefCell::new(None)),
        };

        ast_module.code_gen(&mut module)?;
        module.finalize();

        // load the main function
        module.main = unsafe {
            module
                .engine
                .get_function::<unsafe extern "C" fn() -> i64>(&generic_mangle(&[], "main"))
        }
        .ok();

        Ok(module)
    }

    pub fn exec(&self) -> ExecuteResult<i64> {
        let main = self.main.as_ref().ok_or(ExecuteError)?;
        let result = unsafe { main.call() };
        Ok(result)
    }

    fn finalize(&self) {
        for f in self.functions.values() {
            assert!(f.verify(true));
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
