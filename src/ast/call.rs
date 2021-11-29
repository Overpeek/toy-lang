use super::{Ident, ParseAst, Result, Rule, Type, TypeOf, VisibleVars};
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: Ident,

    ty: Option<Type>,
}

impl ParseAst for Call {
    fn parse(token: Pair<Rule>, vars: &mut VisibleVars) -> Result<Self> {
        assert!(token.as_rule() == Rule::call);

        let name = Ident::parse_single(token.into_inner(), vars)?;
        let ty = vars.get_var(name.value.as_str());

        Ok(Self { name, ty })
    }
}

impl TypeOf for Call {
    fn type_of_checked(&self) -> Option<Type> {
        self.ty
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}()", self.name)
    }
}
