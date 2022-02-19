use super::{Ast, Ident, Result, Rule, Scope, Type};
use pest::{iterators::Pair, Span};
use std::fmt::Display;

//

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionGenInternal<'i> {
    pub name: Ident<'i>,
    pub params: Vec<ParamGen<'i>>,
    pub fn_ty: FnTyGen<'i>,
    pub scope: Scope<'i>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionGen<'i> {
    pub internal: Box<FunctionGenInternal<'i>>,

    span: Span<'i>,
}

impl<'i> Ast<'i> for FunctionGen<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        assert_eq!(token.as_rule(), Rule::function);

        let span = token.as_span();
        let mut tokens = token.into_inner();

        let internal = Box::new(FunctionGenInternal {
            name: Ast::parse(tokens.next().unwrap())?,
            params: if let Some(Rule::param) = tokens.peek().map(|token| token.as_rule()) {
                Ast::parse(tokens.next().unwrap())?
            } else {
                vec![]
            },
            fn_ty: Ast::parse(tokens.next().unwrap())?,
            scope: Ast::parse(tokens.next().unwrap())?,
        });

        Ok(Self { internal, span })
    }
}

/* impl<'i> TypeOf<'i> for FunctionGen<'i> {
    fn type_check_impl(
        &mut self,
        vars: &mut VisibleVars,
        solver: &mut GenericSolver<'i>,
    ) -> Result<()> {
        self.internal.generic = /* matches!(self.internal.fn_ty.ty, Type::Unresolved)
            || */ self
                .internal
                .params
                .iter()
                .any(|param| matches!(param.ty, Type::Unresolved));
        self.ty = Some(self.internal.fn_ty.ty);

        solver.insert(self.internal.name.value.as_str(), self.clone());

        let ty = if self.internal.generic {
            // generic functions are ignored until generated
            self.internal.fn_ty.ty
        } else {
            // other functions are generated immediately
            vars.push();
            self.internal.params.iter().for_each(|param| {
                vars.push_var(param.ident.value.as_str(), param.ty);
            });
            self.internal.scope.type_check(vars, solver)?;
            vars.pop();

            self.internal.scope.type_of()
        };

        let fn_ty = self.internal.fn_ty.ty;
        if fn_ty != Type::Unresolved && ty != fn_ty {
            return Err(Error::new_type_mismatch(self.span(), &fn_ty, &ty));
        }
        self.ty = Some(ty);
        self.internal.fn_ty.ty = ty;

        Ok(())
    }

    fn type_of_impl(&self) -> Option<Type> {
        self.ty
    }
} */

impl<'i> Display for FunctionGen<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}() {}", self.internal.name, self.internal.scope)
    }
}

impl<'i> FunctionGen<'i> {
    /* pub fn global(statements: Vec<Statement<'i>>, span: Span<'i>) -> Self {
        Self {
            internal: Box::new(FunctionInternal {
                name: Ident::new("__global"),
                params: vec![],
                fn_ty: FnTy {
                    span: Span::new("__global", 0, 5).unwrap(),
                    ty: Type::Unresolved,
                },
                scope: Scope::global(statements, span.clone()),
                generic: false,
                generated: vec![],
            }),
            span,
            ty: None,
        }
    } */

    /* pub fn generize(mut self, solver: &mut GenericSolver<'i>) {
        self.ty = Some(self.internal.fn_ty.ty);

        self.internal.generic = self
            .internal
            .params
            .iter()
            .any(|param| matches!(param.ty, Type::Unresolved));

        solver.insert(self.internal.name.value.clone().as_str(), self);
    }

    pub fn generic(&self) -> bool {
        self.internal.generic
    } */
}

//

#[derive(Debug, Clone, PartialEq)]
pub struct ParamGen<'i> {
    pub ident: Ident<'i>,
    pub span: Span<'i>,
    pub ty: Type,
}

impl<'i> Ast<'i> for Vec<ParamGen<'i>> {
    fn span(&self) -> Span<'i> {
        Span::new("unreachable", 0, 11).unwrap()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        // let span = token.as_span();
        // match_rule(&span, token.as_rule(), Rule::params)?;
        let mut tokens = token.into_inner();

        let mut params = vec![];
        while tokens.peek().is_some() {
            let ident = Ident::parse(tokens.next().unwrap())?;
            if let Some(Rule::ty) = tokens.peek().map(|token| token.as_rule()) {
                let ty = Type::parse(tokens.next().unwrap())?;
                let span = ty.span();
                params.push(ParamGen { ident, span, ty })
            } else {
                let span = ident.span();
                params.push(ParamGen {
                    ident,
                    span,
                    ty: Type::Unresolved,
                })
            }
        }

        Ok(params)
    }
}

//

#[derive(Debug, Clone, PartialEq)]
pub struct FnTyGen<'i> {
    pub(super) span: Span<'i>,
    pub(super) ty: Type,
}

impl<'i> Ast<'i> for FnTyGen<'i> {
    fn span(&self) -> Span<'i> {
        self.span.clone()
    }

    fn parse(token: Pair<'i, Rule>) -> Result<Self> {
        assert_eq!(token.as_rule(), Rule::fn_ty);

        let span = token.as_span();
        let mut tokens = token.into_inner();
        Ok(match tokens.peek() {
            Some(_) => Self {
                span,
                ty: Ast::parse(tokens.next().unwrap())?,
            },
            None => Self {
                span,
                ty: Type::Unit,
            },
        })
    }
}
