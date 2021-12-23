use crate::ast;
use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
    passes::{PassManager, PassManagerBuilder},
    values::{BasicValueEnum, FloatValue, FunctionValue},
    FloatPredicate, OptimizationLevel,
};
use std::{cell::RefCell, collections::HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Optimize {
    O0,
    O1,
    O2,
    O3,
}

pub struct Compiler {
    context: Context,
}

pub struct CompilerModule<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    opt: Optimize,
    lpm: PassManager<Module<'ctx>>,
    mpm: PassManager<Module<'ctx>>,
    fpm: PassManager<FunctionValue<'ctx>>,

    engine: ExecutionEngine<'ctx>,
    main: Option<JitFunction<unsafe extern "C" fn() -> f64>>,

    functions: HashMap<String, FunctionValue<'ctx>>,
    function: RefCell<Option<(FunctionValue<'ctx>, HashMap<String, FloatValue<'ctx>>)>>, // current function and values
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

    pub fn module(&self, ast: &ast::Module, opt: Optimize) -> CompilerModule<'_> {
        let context = &self.context;
        let module = context.create_module("repl");
        let builder = context.create_builder();

        let fpmb = PassManagerBuilder::create();
        fpmb.set_optimization_level(match opt {
            Optimize::O0 => OptimizationLevel::None,
            Optimize::O1 => OptimizationLevel::Less,
            Optimize::O2 => OptimizationLevel::Default,
            Optimize::O3 => OptimizationLevel::Aggressive,
        });
        fpmb.set_inliner_with_threshold(100);

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

        let mut module = CompilerModule {
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
            function: RefCell::new(None),
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
        if !matches!(self.opt, Optimize::O0) {
            self.mpm.run_on(&self.module);
            self.lpm.run_on(&self.module);
        }
        log::debug!("LLVM IR: {}", self.module.print_to_string().to_string());
    }

    fn compile_mod(&mut self, module: &ast::Module) {
        log::debug!("compiling module");

        let function_protos: Vec<(FunctionValue, &ast::Function)> = module
            .functions
            .values()
            .map(|function| {
                let name = function.internal.name.value.clone();
                let proto = self.compile_fn_proto(name.as_str());
                self.functions.insert(name, proto);
                (proto, function)
            })
            .collect();

        // fix wrong clippy warning `needless_collect`
        function_protos.len();

        function_protos.into_iter().for_each(|(proto, function)| {
            self.compile_fn(proto, function);
        });
    }

    fn compile_fn(
        &mut self,
        proto: FunctionValue<'ctx>,
        function: &ast::Function,
    ) -> FunctionValue<'ctx> {
        log::debug!("compiling fn: '{}'", function.internal.name);

        // block
        let entry = self.context.append_basic_block(proto, "entry");
        self.builder.position_at_end(entry);

        // scope
        *self.function.borrow_mut() = Some((proto, HashMap::new()));
        let value = self.compile_scope(&function.internal.scope);

        // return
        self.builder.build_return(Some(&value));
        *self.function.borrow_mut() = None;

        // verification
        assert!(proto.verify(true));
        if !matches!(self.opt, Optimize::O0) {
            self.fpm.run_on(&proto);
        }
        proto
    }

    fn compile_fn_proto(&mut self, name: &str) -> FunctionValue<'ctx> {
        let fn_type = self.context.f64_type().fn_type(&[], false);
        self.module.add_function(name, fn_type, None)
    }

    fn compile_scope(&self, scope: &ast::Scope) -> FloatValue<'ctx> {
        scope
            .statements
            .iter()
            .map(|stmt| self.compile_statement(stmt))
            .last()
            .unwrap_or_else(|| self.context.f64_type().const_float(0.0))
    }

    fn compile_statement(&self, statement: &ast::Statement) -> FloatValue<'ctx> {
        match statement.internal.as_ref() {
            ast::StatementInternal::Expr(expr) => self.compile_expr(expr),
            ast::StatementInternal::Assign(assign) => self.compile_assign(assign),
        }
    }

    fn compile_assign(&self, assign: &ast::Assign) -> FloatValue<'ctx> {
        let mut function = self.function.borrow_mut();
        let function = function.as_mut().expect("Assign outside of any function?");

        let value = self.compile_expr(&assign.expr);
        function.1.insert(assign.name.value.clone(), value);
        value
    }

    fn compile_expr(&self, expr: &ast::Expr) -> FloatValue<'ctx> {
        self.compile_any_expr(expr.internal.as_ref())
    }

    fn compile_any_expr(&self, expr: &ast::ExprInternal) -> FloatValue<'ctx> {
        match expr {
            ast::ExprInternal::BinaryExpr(expr) => self.compile_binary_expr(expr),
            ast::ExprInternal::UnaryExpr(expr) => self.compile_unary_expr(expr),
            ast::ExprInternal::Term(term) => self.compile_term(term),
        }
    }

    fn compile_binary_expr(&self, expr: &ast::BinaryExpr) -> FloatValue<'ctx> {
        let lhs = self.compile_expr(&expr.operands.lhs);
        let rhs = self.compile_expr(&expr.operands.rhs);

        match expr.operator {
            ast::BinaryOp::Add => self.builder.build_float_add(lhs, rhs, "BinaryExpr add"),
            ast::BinaryOp::Sub => self.builder.build_float_sub(lhs, rhs, "BinaryExpr sub"),
            ast::BinaryOp::Mul => self.builder.build_float_mul(lhs, rhs, "BinaryExpr mul"),
            ast::BinaryOp::Div => self.builder.build_float_div(lhs, rhs, "BinaryExpr div"),
        }
    }

    fn compile_unary_expr(&self, expr: &ast::UnaryExpr) -> FloatValue<'ctx> {
        let operand = self.compile_expr(expr.operand.as_ref());

        match expr.operator {
            ast::UnaryOp::Plus => operand,
            ast::UnaryOp::Neg => self.builder.build_float_neg(operand, "UnaryExpr neg"),
        }
    }

    fn compile_term(&self, term: &ast::Term) -> FloatValue<'ctx> {
        self.compile_factor(term.internal.as_ref())
    }

    fn compile_factor(&self, factor: &ast::TermInternal) -> FloatValue<'ctx> {
        match factor {
            ast::TermInternal::Lit(lit) => self.compile_lit(lit),
            ast::TermInternal::Expr(expr) => self.compile_expr(expr),
            ast::TermInternal::Branch(branch) => self.compile_branch(branch),
            ast::TermInternal::Access(access) => self.compile_access(access),
            ast::TermInternal::Call(call) => self.compile_call(call),
        }
    }

    fn compile_lit(&self, lit: &ast::Lit) -> FloatValue<'ctx> {
        match lit {
            ast::Lit::F64(v) => self.compile_float(v),
            ast::Lit::I64(v) => self.compile_int(v),
            ast::Lit::Bool(v) => self
                .context
                .f64_type()
                .const_float(if *v { 1.0 } else { 0.0 }),
            ast::Lit::Unit(_) => self.context.f64_type().const_float(0.0),
        }
    }

    fn compile_branch(&self, branch: &ast::Branch) -> FloatValue<'ctx> {
        let cond = self.compile_expr(&branch.internal.test);
        let const_zero = self.context.f64_type().const_zero();
        let function = self.function.borrow();
        let function = function.as_ref().expect("Branch outside of any function?");

        let on_true = self
            .context
            .append_basic_block(function.0, "Branch on_true");
        let on_false = self
            .context
            .append_basic_block(function.0, "Branch on_false");
        let r#continue = self
            .context
            .append_basic_block(function.0, "Branch continue");

        let cmp =
            self.builder
                .build_float_compare(FloatPredicate::ONE, cond, const_zero, "Branch cmp");
        self.builder
            .build_conditional_branch(cmp, on_true, on_false);

        // on_true block
        self.builder.position_at_end(on_true);
        let result_a = self.compile_scope(&branch.internal.on_true);
        self.builder.build_unconditional_branch(r#continue);
        // on_false block
        self.builder.position_at_end(on_false);
        let result_b = self.compile_scope(&branch.internal.on_false);
        self.builder.build_unconditional_branch(r#continue);
        // continue block
        self.builder.position_at_end(r#continue);

        let phi = self
            .builder
            .build_phi(self.context.f64_type(), "Branch phi");
        phi.add_incoming(&[(&result_a, on_true), (&result_b, on_false)]);
        phi.as_basic_value().into_float_value()
    }

    fn compile_access(&self, access: &ast::Access) -> FloatValue<'ctx> {
        let function = self.function.borrow();
        let function = function.as_ref().expect("Access outside of any function?");

        *function
            .1
            .get(access.name.value.as_str())
            .unwrap_or_else(|| panic!("Variable: {} not found", access.name.value.as_str()))
    }

    fn compile_float(&self, float: &f64) -> FloatValue<'ctx> {
        self.context.f64_type().const_float(*float)
    }

    fn compile_int(&self, float: &i64) -> FloatValue<'ctx> {
        self.context.f64_type().const_float(*float as f64) // TODO:
    }

    fn compile_call(&self, call: &ast::Call) -> FloatValue<'ctx> {
        let func = *self
            .functions
            .get(call.name.value.as_str())
            .unwrap_or_else(|| panic!("Unknown function: {}", call.name.value.as_str()));
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
