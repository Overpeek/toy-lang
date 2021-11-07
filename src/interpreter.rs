use std::collections::HashMap;

use crate::ast;

#[derive(Debug, Clone, Default)]
pub struct Memory {
    functions: HashMap<String, ast::Function>,
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

    pub fn exec(&self, memory: &mut Memory, ast: &ast::Ast) -> f64 {
        self.exec_module(memory, &ast.module);

        let main = memory
            .functions
            .get("main")
            .expect("No main function")
            .clone();
        self.exec_function(memory, &main)
    }

    fn exec_module(&self, memory: &mut Memory, module: &ast::Module) {
        for function in module.iter().cloned() {
            memory.functions.insert(function.0.clone(), function);
        }
    }

    fn exec_function(&self, memory: &mut Memory, function: &ast::Function) -> f64 {
        self.exec_scope(memory, &function.1)
    }

    fn exec_scope(&self, memory: &mut Memory, scope: &ast::Scope) -> f64 {
        let mut last = 0.0;
        for statement in scope {
            last = self.exec_statement(memory, statement);
        }
        last
    }

    fn exec_statement(&self, memory: &mut Memory, statement: &ast::Statement) -> f64 {
        self.exec_expr(memory, statement)
    }

    fn exec_expr(&self, memory: &mut Memory, expr: &ast::Expr) -> f64 {
        let mut lhs = self.exec_term(memory, &expr.0);
        for (op, rhs) in &expr.1 {
            let rhs = self.exec_term(memory, &rhs);
            match op {
                ast::Either::A(_) => lhs += rhs,
                ast::Either::B(_) => lhs -= rhs,
            }
        }
        lhs
    }

    fn exec_term(&self, memory: &mut Memory, term: &ast::Term) -> f64 {
        let mut lhs = self.exec_factor(memory, &term.0);
        for (op, rhs) in &term.1 {
            let rhs = self.exec_factor(memory, &rhs);
            match op {
                ast::Either::A(_) => lhs *= rhs,
                ast::Either::B(_) => lhs /= rhs,
            }
        }
        lhs
    }

    fn exec_factor(&self, memory: &mut Memory, factor: &ast::Factor) -> f64 {
        match factor {
            ast::Factor::F64(f) => *f,
            ast::Factor::I64(i) => *i as f64,
            ast::Factor::Expr(expr) => self.exec_expr(memory, expr),
            ast::Factor::Call(call) => self.exec_call(memory, call),
            ast::Factor::Sign(sign) => {
                let rhs = self.exec_factor(memory, &sign.1);
                match sign.0 {
                    ast::Either::A(_) => rhs,
                    ast::Either::B(_) => -rhs,
                }
            }
        }
    }

    fn exec_call(&self, memory: &mut Memory, call: &ast::Call) -> f64 {
        let func = memory
            .functions
            .get(call)
            .expect("Unknown function")
            .clone();
        self.exec_function(memory, &func)
    }
}
