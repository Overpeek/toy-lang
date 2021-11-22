use crate::ast::{
    Access, Assign, BinaryExpr, BinaryOp, Branch, Call, Expr, ExprInternal, Function, Ident, Lit,
    Module, Scope, Statement, Term, TermInternal, UnaryExpr, UnaryOp,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

pub const MAX_CALL_DEPTH: usize = 32;

// ----------
// Error type
// ----------

pub enum Error {
    UnknownVariable(Ident),
    UnknownFunction(Ident),
    MissingMain,
    StackOverflow,
}
pub type Result<T> = ::std::result::Result<T, Error>;

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownVariable(err) => {
                write!(f, "Cannot find variable '{}' in this scope", err)
            }
            Error::UnknownFunction(err) => {
                write!(f, "Cannot find function '{}' in this scope", err)
            }
            Error::MissingMain => write!(f, "No main function"),
            Error::StackOverflow => write!(f, "Tried to overflow its max stack size"),
        }
    }
}

// -----------
// Interpreter
// -----------

pub trait Interpreter {
    fn eval(&self) -> Result<f64>;
}

impl Interpreter for Module {
    fn eval(&self) -> Result<f64> {
        let mut memory = ScopeMemory::default();
        self.eval_with_memory(&mut memory, self)
    }
}

#[derive(Debug, Clone, Default)]
struct ScopeMemory {
    variables: HashMap<String, f64>,
}

trait InterpreterWithMemory {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64>;
}

impl InterpreterWithMemory for Module {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        self.functions
            .get("main")
            .ok_or(Error::MissingMain)?
            .eval_with_memory(memory, module)
    }
}

impl InterpreterWithMemory for Function {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        self.internal.scope.eval_with_memory(memory, module)
    }
}

impl InterpreterWithMemory for Scope {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        let mut last = 0.0;
        for statement in self.statements.iter() {
            last = statement.eval_with_memory(memory, module)?;
        }
        Ok(last)
    }
}

impl InterpreterWithMemory for Statement {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        match self {
            Statement::Expr(expr) => expr.eval_with_memory(memory, module),
            Statement::Assign(assign) => assign.eval_with_memory(memory, module),
        }
    }
}

impl InterpreterWithMemory for Assign {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        let expr = self.expr.eval_with_memory(memory, module)?;
        if memory.variables.insert(self.name.clone(), expr).is_some() {
            log::debug!("shadowing variable: '{}'", self.name);
        }
        Ok(0.0)
    }
}

impl InterpreterWithMemory for Expr {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        match self.internal.as_ref() {
            ExprInternal::BinaryExpr(expr) => expr.eval_with_memory(memory, module),
            ExprInternal::UnaryExpr(expr) => expr.eval_with_memory(memory, module),
            ExprInternal::Term(term) => term.eval_with_memory(memory, module),
        }
    }
}

impl InterpreterWithMemory for BinaryExpr {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        let sides = self.operands.as_ref();
        let (lhs, rhs) = (
            sides.lhs.eval_with_memory(memory, module)?,
            sides.rhs.eval_with_memory(memory, module)?,
        );

        Ok(match self.operator {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div => lhs / rhs,
        })
    }
}

impl InterpreterWithMemory for UnaryExpr {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        let operand = self.operand.eval_with_memory(memory, module)?;
        Ok(match self.operator {
            UnaryOp::Plus => operand,
            UnaryOp::Neg => -operand,
        })
    }
}

impl InterpreterWithMemory for Term {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        match self.internal.as_ref() {
            TermInternal::Lit(v) => v as &dyn InterpreterWithMemory,
            TermInternal::Expr(v) => v as _,
            TermInternal::Branch(v) => v as _,
            TermInternal::Access(v) => v as _,
            TermInternal::Call(v) => v as _,
        }
        .eval_with_memory(memory, module)
    }
}

impl InterpreterWithMemory for Lit {
    fn eval_with_memory(&self, _: &mut ScopeMemory, _: &Module) -> Result<f64> {
        Ok(match self {
            Lit::I64(i) => *i as f64,
            Lit::F64(f) => *f,
            Lit::Bool(_) => 0.0,
            Lit::Unit(_) => 0.0,
        })
    }
}

impl InterpreterWithMemory for Call {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        module
            .functions
            .get(self.name.as_str())
            .ok_or_else(|| Error::UnknownFunction(self.name.clone()))?
            .eval_with_memory(memory, module)
    }
}

impl InterpreterWithMemory for Branch {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, module: &Module) -> Result<f64> {
        let test = self.internal.test.eval_with_memory(memory, module)?;
        if (test).abs() > std::f64::EPSILON {
            &self.internal.on_true
        } else {
            &self.internal.on_false
        }
        .eval_with_memory(memory, module)
    }
}

impl InterpreterWithMemory for Access {
    fn eval_with_memory(&self, memory: &mut ScopeMemory, _: &Module) -> Result<f64> {
        memory
            .variables
            .get(self.name.as_str())
            .cloned()
            .ok_or_else(|| Error::UnknownVariable(self.name.clone()))
    }
}

/* pub type StackFrame = HashMap<Ident, f64>;

#[derive(Debug, Clone)]
pub struct Memory {
    functions: HashMap<Ident, ast::Function>,
    stack_frames: Vec<
        // closed stack frames (earlier functions)
        Vec<
            // open stack frames (earlier scopes)
            StackFrame,
        >,
    >,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            functions: Default::default(),
            stack_frames: vec![vec![Default::default()]],
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_closed(&mut self) -> Result<()> {
        if self.stack_frames.len() < MAX_CALL_DEPTH {
            self.stack_frames.push(vec![Default::default()]);
            Ok(())
        } else {
            Err(Error::StackOverflow)
        }
    }

    pub fn push(&mut self) {
        self.stack_frames
            .last_mut()
            .unwrap()
            .push(Default::default());
    }

    pub fn pop_closed(&mut self) {
        let last = &mut self.stack_frames;
        if last.len() != 1 {
            last.pop();
        }
    }

    pub fn pop(&mut self) {
        let last = self.stack_frames.last_mut().unwrap();
        if last.len() != 1 {
            last.pop();
        }
    }

    pub fn insert(&mut self, ident: Ident, var: f64) {
        self.stack_frames
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .insert(ident, var);
    }

    pub fn get(&self, ident: &Ident) -> Option<&f64> {
        self.stack_frames
            .last()
            .unwrap()
            .iter()
            .rev()
            .find_map(|frame| frame.get(ident))
    }

    pub fn get_mut(&mut self, ident: &Ident) -> Option<&mut f64> {
        self.stack_frames
            .last_mut()
            .unwrap()
            .iter_mut()
            .rev()
            .find_map(|frame| frame.get_mut(ident))
    }
}

pub struct Interpreter;

impl Default for Interpreter {
    fn default() -> Self {
        Self {}
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exec(&self, memory: &mut Memory, ast: &ast::Ast) -> Result<f64> {
        self.exec_module(memory, &ast.module)?;

        let main = memory
            .functions
            .get("main")
            .expect("No main function")
            .clone();
        self.exec_function(memory, &main, &vec![])
    }

    fn exec_module(&self, memory: &mut Memory, module: &ast::Module) -> Result<()> {
        for function in module.iter().cloned() {
            memory.functions.insert(function.0.clone(), function);
        }
        Ok(())
    }

    fn exec_function(
        &self,
        memory: &mut Memory,
        function: &ast::Function,
        args: &ast::Args,
    ) -> Result<f64> {
        assert!(
            function.1.len() == args.len(),
            "Function params and arguments given do not match"
        );

        let values: Vec<_> = args
            .iter()
            .map(|arg| self.exec_statement(memory, arg))
            .collect::<Result<_>>()?;

        memory.push_closed()?;
        values
            .into_iter()
            .zip(function.1.iter())
            .for_each(|(arg, param)| memory.insert(param.clone(), arg));
        let result = self.exec_scope(memory, &function.2)?;
        memory.pop_closed();

        Ok(result)
    }

    fn exec_scope(&self, memory: &mut Memory, scope: &ast::Scope) -> Result<f64> {
        let mut last = 0.0;
        memory.push();
        for statement in scope {
            last = self.exec_statement(memory, statement)?;
        }
        // log::debug!("Memory: {:?}", memory);
        memory.pop();
        Ok(last)
    }

    fn exec_statement(&self, memory: &mut Memory, statement: &ast::Statement) -> Result<f64> {
        match statement {
            ast::Statement::Expr(expr) => self.exec_expr(memory, expr),
            ast::Statement::NoReturn(no_return) => self.exec_no_return(memory, no_return),
        }
    }

    fn exec_no_return(&self, memory: &mut Memory, no_return: &ast::NoReturn) -> Result<f64> {
        match no_return {
            ast::NoReturn::Assign(assign) => self.exec_assign(memory, assign),
        }
    }

    fn exec_expr(&self, memory: &mut Memory, expr: &ast::Expr) -> Result<f64> {
        match expr {
            ast::Expr::ArithExpr(arith_expr) => self.exec_arith_expr(memory, arith_expr),
            ast::Expr::Branch(branch) => self.exec_branch(memory, branch),
        }
    }

    fn exec_access(&self, memory: &mut Memory, access: &ast::Access) -> Result<f64> {
        memory
            .get(access)
            .cloned()
            .ok_or_else(|| Error::UnknownVariable(access.clone()))
    }

    fn exec_branch(&self, memory: &mut Memory, branch: &ast::Branch) -> Result<f64> {
        let eval = self.exec_statement(memory, &branch.0)?;
        if eval == 1.0 {
            self.exec_scope(memory, &branch.1)
        } else {
            self.exec_scope(memory, &branch.2)
        }
    }

    fn exec_assign(&self, memory: &mut Memory, assign: &ast::Assign) -> Result<f64> {
        let value = self.exec_statement(memory, &assign.1)?;
        memory.insert(assign.0.clone(), value);
        Ok(0.0)
    }

    fn exec_arith_expr(&self, memory: &mut Memory, arith_expr: &ast::ArithExpr) -> Result<f64> {
        let mut lhs = self.exec_arith_term(memory, &arith_expr.0)?;
        for (op, rhs) in &arith_expr.1 {
            let rhs = self.exec_arith_term(memory, &rhs)?;
            match op {
                ast::Either::A(_) => lhs += rhs,
                ast::Either::B(_) => lhs -= rhs,
            }
        }
        Ok(lhs)
    }

    fn exec_arith_term(&self, memory: &mut Memory, arith_term: &ast::ArithTerm) -> Result<f64> {
        let mut lhs = self.exec_factor(memory, &arith_term.0)?;
        for (op, rhs) in &arith_term.1 {
            let rhs = self.exec_factor(memory, rhs)?;
            match op {
                ast::Either::A(_) => lhs *= rhs,
                ast::Either::B(_) => lhs /= rhs,
            }
        }
        Ok(lhs)
    }

    fn exec_factor(&self, memory: &mut Memory, factor: &ast::Factor) -> Result<f64> {
        match factor {
            ast::Factor::F64(f) => Ok(*f),
            ast::Factor::I64(i) => Ok(*i as f64),
            ast::Factor::Statement(statement) => self.exec_statement(memory, statement),
            ast::Factor::Call(call) => self.exec_call(memory, call),
            ast::Factor::Sign(sign) => {
                let rhs = self.exec_factor(memory, &sign.1);
                match sign.0 {
                    ast::Either::A(_) => rhs,
                    ast::Either::B(_) => rhs.map(|rhs| -rhs),
                }
            }
            ast::Factor::Access(access) => self.exec_access(memory, access),
            ast::Factor::Branch(branch) => self.exec_branch(memory, branch),
        }
    }

    fn exec_call(&self, memory: &mut Memory, call: &ast::Call) -> Result<f64> {
        let func = memory
            .functions
            .get(&call.0)
            .expect("Unknown function")
            .clone();
        self.exec_function(memory, &func, &call.1)
    }
}
 */
