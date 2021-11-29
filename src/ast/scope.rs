use super::{ParseAst, Result, Rule, Statement, StatementInternal, Type, TypeOf, VisibleVars};
use pest::iterators::Pair;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub statements: Vec<Statement>,

    alloc: HashMap<String, Type>,
    ty: Option<Type>,
}

impl ParseAst for Scope {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::scope);

        // all statements within this scope
        vars.push();
        let statements: Result<Vec<Statement>> = token
            .into_inner()
            .map(|token| ParseAst::parse(token, vars))
            .collect();
        vars.pop();
        let statements = statements?;

        // assign statements are the only statements to allocate
        let alloc = statements
            .iter()
            .filter_map(|statement| match statement.internal.as_ref() {
                StatementInternal::Assign(assign) => {
                    Some((assign.name.value.clone(), assign.expr.type_of()))
                }
                _ => None,
            })
            .collect();

        let ty = statements
            .last()
            .as_ref()
            .map_or(Some(Type::Unit), |stmt| stmt.type_of_checked());

        Ok(Scope {
            statements,
            alloc,
            ty,
        })
    }
}

impl TypeOf for Scope {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for statement in self.statements.iter() {
            statement.fmt(f)?;
        }
        write!(f, "}}")
    }
}
