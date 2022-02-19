use super::{Ast, Function, FunctionGen, Result, Rule, Statement, Type, TypeOf, VisibleVars};
use crate::ast::match_rule;
use itertools::{Either, Itertools};
use pest::{iterators::Pair, Span};
use std::{collections::HashMap, fmt::Display};

//

#[derive(Debug, Clone)]
pub struct Module<'i> {
    pub function_gens: HashMap<String, FunctionGen<'i>>,
    pub functions: HashMap<String, Function<'i>>,
    global: Vec<Statement<'i>>,

    span: Span<'i>,
}

/* #[derive(Debug, Clone)]
enum Global<'i> {
    Function(Function<'i>),
    Statements(Vec<Statement<'i>>),
} */

//

impl<'i> Ast<'i> for Module<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::module)?;
        let tokens = token.into_inner();

        let (global, functions): (Vec<Result<_>>, Vec<Result<_>>) =
            tokens.partition_map(|token| match token.as_rule() {
                Rule::statement => Either::Left(Statement::parse(token)),
                Rule::function => Either::Right(FunctionGen::parse(token)),
                _ => unreachable!(),
            });

        let global = global.into_iter().collect::<Result<Vec<_>>>()?;
        let functions = functions.into_iter().collect::<Result<Vec<_>>>()?;
        let (functions, function_gens): (Vec<Function>, Vec<FunctionGen>) = functions
            .into_iter()
            .partition_map(|f| match Function::new_non_generic(f) {
                Ok(f) => Either::Left(f),
                Err(f) => Either::Right(f),
            });

        let functions = functions
            .into_iter()
            .map(|f| (f.internal.name.value.clone(), f))
            .collect();
        let function_gens = function_gens
            .into_iter()
            .map(|f| (f.internal.name.value.clone(), f))
            .collect();

        Ok(Self {
            function_gens,
            functions,
            global,

            span,
        })
    }
}

impl<'i> TypeOf<'i> for Module<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        let mut statements = vec![];
        std::mem::swap(&mut statements, &mut self.global);

        for (_, f) in self.function_gens.drain() {
            vars.push_fn_gen(&f.internal.name.value.clone(), f);
        }

        for (_, f) in self.functions.drain() {
            let sig: Box<[Type]> = f.internal.params.iter().map(|param| param.ty).collect();
            vars.push_fn(&f.internal.name.value.clone(), &sig, f);
        }

        let global = Function::global(vars, statements, self.span())?;
        let sig: Box<[Type]> = global
            .internal
            .params
            .iter()
            .map(|param| param.ty)
            .collect();
        vars.push_fn(&global.internal.name.value.clone(), &sig, global);

        self.function_gens.extend(vars.function_gens.drain());
        self.functions.extend(vars.functions.drain());

        for (name, f) in self.functions.iter_mut() {
            f.internal.name.value = name.clone();
        }

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        Some(Type::Unresolved)
    }
}

impl<'i> Display for Module<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, function) in self.function_gens.iter() {
            function.fmt(f)?;
        }
        for (_, function) in self.functions.iter() {
            function.fmt(f)?;
        }
        Ok(())
    }
}

impl<'i> Module<'i> {
    pub fn take_functions(&mut self) -> HashMap<String, Function<'i>> {
        let mut tmp = Default::default();
        std::mem::swap(&mut tmp, &mut self.functions);
        tmp
    }
}
