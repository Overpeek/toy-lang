use super::{ParseAst, Result, Rule, Statement, Type, TypeOf};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub statements: Vec<Statement>,
    pub ty: Type,
}

impl ParseAst for Scope {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::scope);

        let statements: Vec<Statement> = token
            .into_inner()
            .map(ParseAst::parse)
            .collect::<Result<_>>()?;
        let ty = statements
            .last()
            .map_or(Type::Unit, |statement| statement.type_of());
        Ok(Scope { statements, ty })
    }
}

impl TypeOf for Scope {
    fn type_of(&self) -> Type {
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
