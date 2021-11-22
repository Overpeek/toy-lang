use super::{function::Function, ParseAst, Result, Rule};
use pest::iterators::Pair;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub struct Module {
    pub functions: HashMap<String, Function>,
}

impl ParseAst for Module {
    fn parse(token: Pair<Rule>) -> Result<Self> {
        assert!(token.as_rule() == Rule::module);

        Ok(Self {
            functions: token
                .into_inner()
                .map(|token| {
                    let func: Function = ParseAst::parse(token)?;
                    Ok((func.internal.name.clone(), func))
                })
                .collect::<Result<_>>()?,
        })
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, function) in self.functions.iter() {
            function.fmt(f)?;
        }
        Ok(())
    }
}
