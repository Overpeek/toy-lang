use backtrace::Backtrace;
use pest::{
    error::{Error as PestError, ErrorVariant},
    iterators::{Pair, Pairs},
    Parser, Span,
};
use std::{
    collections::hash_map::{DefaultHasher, Entry},
    hash::{Hash, Hasher},
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

pub use self::access::*;
pub use self::assign::*;
pub use self::binary::*;
pub use self::branch::*;
pub use self::call::*;
pub use self::expr::*;
pub use self::function::*;
pub use self::ident::*;
pub use self::module::*;
pub use self::r#type::*;
pub use self::scope::*;
pub use self::statement::*;
pub use self::term::*;
pub use self::unary::*;

pub mod access;
pub mod assign;
pub mod binary;
pub mod branch;
pub mod call;
pub mod expr;
pub mod function;
pub mod ident;
pub mod module;
pub mod scope;
pub mod statement;
pub mod term;
pub mod r#type;
pub mod unary;

// ------
// Parser
// ------

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ToyLangParser;

pub fn parse<'i>(input: &'i str) -> Result<Module<'i>> {
    let mut tokens = ToyLangParser::parse(Rule::input, input).map_err(Error::new_pest)?;
    let mut module = Module::<'i>::parse(tokens.next().unwrap())?;

    let mut vars = VisibleVars::new();
    let mut solver = GenericSolver::new();
    module.type_check(&mut vars, &mut solver)?;

    solver.get(
        &mut vars,
        Span::new("unreachable", 0, 11).unwrap(),
        "main",
        &[],
    )?;

    module.functions.clear();
    for (name, f) in solver.evaluated_generics.drain() {
        // let name = generic_mangle(&sig, f.internal.name.value.as_str());
        let mut f = match f {
            FnOrTy::Fn(f) => f,
            FnOrTy::Ty(_) => unreachable!(),
        };
        f.internal.name.value = name.clone();
        module.functions.insert(name, f);
    }

    Ok(module)
}

pub fn generic_mangle(sig: &[Type], name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    sig.hash(&mut hasher);
    let id = hasher.finish();

    format!("__[{name}]__[{id}]")
}

/* pub fn parse_file<P: AsRef<Path>>(path: P) -> ParseFileResult {
    let file = std::fs::read_to_string(path).map_err(ParseFileError::IoError)?;
    parse(&file).map_err(ParseFileError::ParseError)
} */

//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnSig {
    arg_ty: Box<[Type]>,
    out_ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleVars {
    vars: Vec<HashMap<String, Type>>,
}

impl Default for VisibleVars {
    fn default() -> Self {
        Self {
            vars: vec![Default::default()],
        }
    }
}

impl VisibleVars {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_var(&mut self, name: &str, ty: Type) {
        self.vars
            .last_mut()
            .as_mut()
            .unwrap()
            .insert(name.into(), ty);
    }

    pub fn get_var(&self, name: &str) -> Option<Type> {
        self.vars
            .iter()
            .rev()
            .find_map(|map| map.get(name))
            .cloned()
    }

    pub fn push(&mut self) {
        self.vars.push(Default::default())
    }

    pub fn pop(&mut self) {
        self.vars.pop();
    }
}

// ----------
// Error type
// ----------

/* pub enum ParseFileError {
    IoError(std::io::Error),
    ParseError(Error),
}

pub type ParseFileResult = ::std::result::Result<Module, ParseFileError>;

impl Debug for ParseFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for ParseFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(err) => Display::fmt(err, f),
            Self::IoError(err) => Display::fmt(err, f),
        }
    }
} */

pub struct Error {
    error: PestError<Rule>,
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn new_spanned<S: Into<String>>(span: Span, message: S) -> Self {
        Self {
            error: PestError::new_from_span(
                ErrorVariant::CustomError {
                    message: message.into(),
                },
                span,
            ),
        }
    }

    pub fn new_pest(error: PestError<Rule>) -> Self {
        Self { error }
    }

    pub fn new_leftover_tokens(span: Span, token: Pair<Rule>) -> Self {
        Self::new_spanned(span, format!("unexpected token: '{}'", token.as_str()))
    }

    pub fn new_invalid_unary_op(span: Span, op: UnaryOp, ty: Type) -> Self {
        Self::new_spanned(
            span,
            format!(
                "unary operator: '{}' cannot be applied to type: '{}'",
                op, ty
            ),
        )
    }

    pub fn new_invalid_binary_op(span: Span, lhs: Type, op: BinaryOp, rhs: Type) -> Self {
        Self::new_spanned(
            span,
            format!(
                "binary operator: '{}' cannot be applied to lhs: '{}' and rhs: '{}'",
                op, lhs, rhs
            ),
        )
    }

    pub fn new_type_mismatch(span: Span, expect: &Type, got: &Type) -> Self {
        Self::new_spanned(
            span,
            format!("expected type: '{}' but got: '{}'", expect, got),
        )
    }

    pub fn new_rule_mismatch(span: Span, expect: &[Rule], got: Rule) -> Self {
        Self::new_spanned(
            span,
            format!(
                "expected rule: '{}' but got: '{:?}'. Internal error {:?}",
                expect
                    .iter()
                    .map(|rule| format!("{:?}", rule))
                    .collect::<String>(),
                got,
                Backtrace::new()
            ),
        )
    }

    pub fn new_var_not_found(span: Span, expect: &str) -> Self {
        Self::new_spanned(
            span,
            format!("variable '{}' not found within accessible scopes", expect),
        )
    }

    pub fn new_fn_not_found(span: Span, expect: &str) -> Self {
        Self::new_spanned(
            span,
            format!("function '{}' not found within accessible scopes", expect),
        )
    }

    pub fn new_not_callable(span: Span, expect: &str) -> Self {
        Self::new_spanned(span, format!("variable '{}' is not callable", expect))
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

fn match_rule(span: &Span, got: Rule, expect: Rule) -> Result<()> {
    if got == expect {
        Ok(())
    } else {
        Err(Error::new_rule_mismatch(span.clone(), &[expect], got))
    }
}

fn match_any_rule(span: &Span, got: Rule, expect: &[Rule]) -> Result<()> {
    if expect.contains(&got) {
        Ok(())
    } else {
        Err(Error::new_rule_mismatch(span.clone(), expect, got))
    }
}

// -----------
// Parse trait
// -----------

/* struct TokenStream<'i, 'vars> {
    tokens: Pairs<'i, Rule>,
    vars: &'vars mut VisibleVars,
    span: Span<'i>,
}

impl<'i, 'vars> TokenStream<'i, 'vars> {
    fn parse<T>(&mut self) -> Result<T>
    where
        T: Parse,
    {
        let token = match self.tokens.next() {
            Some(token) => token,
            None => return Err(Error::new_spanned(self.span.clone(), "EOI")),
        };
        Parse::parse(token, self.vars)
    }

    fn is_empty(&self) -> bool {
        self.tokens.peek().is_none()
    }
} */

/* trait Parse
where
    Self: Sized,
{
    fn parse(tokens: TokenStream) -> Result<Self>;
} */

trait Ast<'i>
where
    Self: Sized,
{
    fn span(&self) -> Span<'i>;

    fn parse(token: Pair<'i, Rule>) -> Result<Self>;

    fn parse_spanned(token: Pair<'i, Rule>) -> Result<(Self, Span<'i>)> {
        let span = token.as_span();
        Ok((Self::parse(token)?, span))
    }

    fn parse_single(mut tokens: Pairs<'i, Rule>) -> Result<Self> {
        let result = Self::parse(tokens.next().unwrap());
        if let Some(token) = tokens.next() {
            Err(Error::new_leftover_tokens(token.as_span(), token))
        } else {
            result
        }
    }

    fn parse_multiple(tokens: Pairs<'i, Rule>) -> Result<Vec<Self>> {
        tokens.into_iter().map(|token| Self::parse(token)).collect()
    }
}

//

pub trait TypeOf<'i> {
    #[deprecated]
    fn type_check_impl(
        &mut self,
        vars: &mut VisibleVars,
        solver: &mut GenericSolver<'i>,
    ) -> Result<()>;

    #[deprecated]
    fn type_of_impl(&self) -> Option<Type>;

    fn type_check(&mut self, vars: &mut VisibleVars, solver: &mut GenericSolver<'i>) -> Result<()> {
        #[allow(deprecated)]
        let _ = self.type_check_impl(vars, solver)?;

        let _ = self.type_of();

        Ok(())
    }

    fn type_of(&self) -> Type {
        #[allow(deprecated)]
        self.type_of_impl().expect("Wasn't type checked")
    }
}

// --------
// Generics
// --------

pub trait Generic {
    fn eval(self, solver: &mut GenericSolver) -> Result<Type>;
}

#[derive(Debug)]
enum FnOrTy<'i> {
    Fn(Function<'i>),
    Ty(Type),
}

#[derive(Debug, Default)]
pub struct GenericSolver<'i> {
    generics: HashMap<String, Function<'i>>,
    evaluated_generics: HashMap<String, FnOrTy<'i>>,
}

impl<'i> GenericSolver<'i> {
    fn new() -> Self {
        Self::default()
    }

    fn get(
        &mut self,
        vars: &mut VisibleVars,
        span: Span,
        name: &str,
        sig: &[Type],
    ) -> Result<Type> {
        let as_generic = generic_mangle(sig, name);

        if let Some(f) = self.evaluated_generics.get(&as_generic) {
            return Ok(match f {
                FnOrTy::Fn(f) => f.type_of(),
                FnOrTy::Ty(ty) => *ty,
            });
        }

        let mut f = if let Some(f) = self.generics.get(name) {
            f.clone()
        } else {
            return Err(Error::new_fn_not_found(span, name));
        };
        let ty = f.type_of();

        self.evaluated_generics
            .entry(as_generic.clone())
            .or_insert(FnOrTy::Ty(ty));

        assert_eq!(f.internal.params.len(), sig.len());
        f.internal
            .params
            .iter_mut()
            .zip(sig)
            .for_each(|(param, ty)| param.ty = *ty);
        f.type_check(vars, self)?;

        *self.evaluated_generics.get_mut(&as_generic).unwrap() = FnOrTy::Fn(f);

        Ok(ty)
    }

    fn insert(&mut self, name: &str, f: Function<'i>) {
        self.generics.insert(name.into(), f);
        /* if f.generic() {
            self.generics.insert(name.into(), f);
        } else {
            let sig: Box<[Type]> = f.internal.params.iter().map(|param| param.ty).collect();
            let as_generic = generic_mangle(&sig, name);
            self.evaluated_generics.insert(as_generic, FnOrTy::Fn(f));
        } */
    }
}

// pub type GenericSolver<'i> = HashMap<String, Function<'i>>;
