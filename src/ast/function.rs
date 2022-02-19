use super::{FunctionGen, Ident, ParamGen, Scope, Statement, Type, VisibleVars};
use crate::ast::{generic_mangle, Error, Result, TypeOf};
use pest::Span;
use std::fmt::Display;

//

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionInternal<'i> {
    pub name: Ident<'i>,
    pub params: Vec<Param<'i>>,
    pub fn_ty: FnTy<'i>,
    pub scope: Scope<'i>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function<'i> {
    pub internal: Box<FunctionInternal<'i>>,
    pub type_checking: bool,

    span: Span<'i>,
    ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param<'i> {
    pub ident: Ident<'i>,
    pub span: Span<'i>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnTy<'i> {
    span: Span<'i>,
    ty: Type,
}

//

impl<'i> Function<'i> {
    pub fn new_non_generic(gen: FunctionGen<'i>) -> std::result::Result<Self, FunctionGen> {
        if gen.internal.fn_ty.ty == Type::Unresolved {
            return Err(gen);
        }

        let sig: Box<[Type]> = gen.internal.params.iter().map(|param| param.ty).collect();

        if sig.iter().any(|ty| matches!(ty, Type::Unresolved)) {
            return Err(gen);
        }

        let params = gen
            .internal
            .params
            .into_iter()
            .map(|param| {
                let ParamGen { ident, span, ty } = param;
                Param { ident, span, ty }
            })
            .collect();

        let span = Span::new("", 0, 0).unwrap();
        let ty = gen.internal.fn_ty.ty;
        Ok(Self {
            internal: Box::new(FunctionInternal {
                name: gen.internal.name,
                params,
                fn_ty: FnTy {
                    span: gen.internal.fn_ty.span,
                    ty: gen.internal.fn_ty.ty,
                },
                scope: gen.internal.scope,
            }),
            type_checking: false,
            span,
            ty,
        })
    }

    pub fn new(
        vars: &mut VisibleVars<'i>,
        call_site: Span,
        name: &str,
        sig: &[Type],
    ) -> Result<Self> {
        let gen = vars.get_gen_fn(call_site.clone(), name)?;

        // unresolved arg types not allowed
        assert!(sig.iter().all(|ty| !matches!(ty, Type::Unresolved)));
        // arg count must match sig
        let expect = gen.internal.params.len();
        let got = sig.len();
        if expect != got {
            return Err(Error::new_argc_mismatch(call_site, expect, got));
        }

        let params: Vec<Param> = gen
            .internal
            .params
            .iter()
            .zip(sig.iter())
            .map(|(param, &ty)| Param {
                ident: param.ident.clone(),
                span: param.span.clone(),
                ty,
            })
            .collect();

        let mut scope = gen.internal.scope.clone();

        let name = gen.internal.name.value.clone();

        // type checking

        vars.push();
        params.iter().for_each(|param| {
            vars.push_var(param.ident.value.as_str(), param.ty);
        });
        scope.type_check(vars)?;
        vars.pop();

        let ty = scope.type_of();

        let span = Span::new("", 0, 0).unwrap();
        Ok(Self {
            internal: Box::new(FunctionInternal {
                name: Ident::from(generic_mangle(sig, &name), span.clone()),
                params,
                fn_ty: FnTy {
                    span: span.clone(),
                    ty,
                },
                scope,
            }),
            type_checking: false,
            span,
            ty,
        })
    }

    pub fn global(
        vars: &mut VisibleVars<'i>,
        statements: Vec<Statement<'i>>,
        span: Span<'i>,
    ) -> Result<Self> {
        let mut scope = Scope::global(statements, span.clone());
        scope.type_check(vars)?;
        let ty = scope.type_of();
        Ok(Self {
            internal: Box::new(FunctionInternal {
                name: Ident::new("__global"),
                params: vec![],
                fn_ty: FnTy {
                    span: Span::new("__global", 0, 5).unwrap(),
                    ty,
                },
                scope,
            }),
            type_checking: false,
            span,
            ty,
        })
    }
}

impl<'i> TypeOf<'i> for Function<'i> {
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        vars.push();
        self.internal.params.iter().for_each(|param| {
            vars.push_var(param.ident.value.as_str(), param.ty);
        });
        self.internal.scope.type_check(vars)?;
        vars.pop();

        let expect = self.ty;
        let got = self.internal.scope.type_of();
        if expect != got {
            Err(Error::new_type_mismatch(self.span.clone(), &expect, &got))
        } else {
            Ok(())
        }
    }

    fn type_of(&self) -> Type {
        self.ty
    }

    fn type_of_impl(&self) -> Option<Type> {
        Some(self.type_of())
    }
}

impl<'i> Display for Function<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}() {}", self.internal.name, self.internal.scope)
    }
}
