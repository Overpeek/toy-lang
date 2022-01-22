use super::{Ast, GenericSolver, Result, Rule, Statement, Type, TypeOf, VisibleVars};
use crate::ast::match_rule;
use pest::{iterators::Pair, Span};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope<'i> {
    pub statements: Vec<Statement<'i>>,

    /* alloc: HashMap<String, Type>, */
    span: Span<'i>,
    ty: Option<Type>,
}

impl<'i> Ast<'i> for Scope<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        let span = token.as_span();
        match_rule(&span, token.as_rule(), Rule::scope)?;

        // all statements within this scope
        let statements: Vec<Statement> =
            token.into_inner().map(Ast::parse).collect::<Result<_>>()?;

        /* // assign statements are the only statements to allocate
        let alloc = statements
            .iter()
            .filter_map(|statement| match statement.internal.as_ref() {
                StatementInternal::Assign(assign) => {
                    Some((assign.name.value.clone(), assign.expr.type_of()))
                }
                _ => None,
            })
            .collect(); */

        Ok(Scope {
            statements,
            /* alloc, */
            span,
            ty: None,
        })
    }
}

impl<'i> TypeOf<'i> for Scope<'i> {
    fn type_check_impl(
        &mut self,
        vars: &mut VisibleVars,
        solver: &mut GenericSolver<'i>,
    ) -> Result<()> {
        vars.push();
        self.statements
            .iter_mut()
            .try_for_each(|stmt| stmt.type_check(vars, solver))?;
        vars.pop();

        let ty = self
            .statements
            .last()
            .as_ref()
            .map_or(Type::Unit, |stmt| stmt.type_of());
        self.ty = Some(ty);

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
}

impl<'i> Display for Scope<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for statement in self.statements.iter() {
            statement.fmt(f)?;
        }
        write!(f, "}}")
    }
}
