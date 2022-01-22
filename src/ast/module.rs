use crate::ast::match_rule;

use super::{function::Function, Ast, GenericSolver, Result, Rule, Type, TypeOf, VisibleVars};
use pest::{iterators::Pair, Span};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub struct Module<'i> {
    pub functions: HashMap<String, Function<'i>>,

    span: Span<'i>,
}

impl<'i> Ast<'i> for Module<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::module)?;
        let tokens = token.into_inner();

        Ok(Self {
            functions: tokens
                .map(|token| {
                    let func = Function::parse(token)?;
                    Ok((func.internal.name.clone().value, func))
                })
                .collect::<Result<_>>()?,
            span,
        })
    }
}

impl<'i> TypeOf<'i> for Module<'i> {
    fn type_check_impl(
        &mut self,
        vars: &mut VisibleVars,
        solver: &mut GenericSolver<'i>,
    ) -> Result<()> {
        for function in self.functions.values_mut() {
            function.partial_type_check(solver);
        }

        for function in self.functions.values_mut() {
            function.type_check(vars, solver)?;
        }

        Ok(())
    }

    fn type_of_impl(&self) -> Option<super::Type> {
        Some(Type::Unresolved)
    }
}

impl<'i> Display for Module<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, function) in self.functions.iter() {
            function.fmt(f)?;
        }
        Ok(())
    }
}
