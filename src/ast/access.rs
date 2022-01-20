use super::{Ident, ParseAst, Result, Rule, Type, TypeOf, VisibleVars};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Access {
    pub name: Ident,

    ty: Option<Type>,
}

impl ParseAst for Access {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::access);

        let name = Ident::parse_single(token.into_inner(), vars)?;
        let ty = vars.get_var(name.value.as_str());

        Ok(Self { name, ty })
    }
}

impl TypeOf for Access {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}
