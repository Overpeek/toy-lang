use backtrace::Backtrace;
use pest::{
    error::{Error as PestError, ErrorVariant},
    iterators::{Pair, Pairs},
    Parser, Span,
};
use std::{
    collections::hash_map::DefaultHasher,
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
pub use self::function_gen::*;
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
pub mod function_gen;
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
    module.type_check(&mut vars)?;

    Ok(module)
}

pub fn generic_mangle(sig: &[Type], name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    sig.hash(&mut hasher);
    let id = hasher.finish();

    format!("__[{name}]__[{id}]")
}

//

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnSig {
    arg_ty: Box<[Type]>,
    out_ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisibleVars<'i> {
    vars: Vec<HashMap<String, Type>>,
    function_gens: HashMap<String, FunctionGen<'i>>,
    functions: HashMap<String, Function<'i>>,

    fn_ty_cache: HashMap<String, Type>,
}

impl<'i> Default for VisibleVars<'i> {
    fn default() -> Self {
        Self {
            vars: vec![Default::default()],
            function_gens: Default::default(),
            functions: Default::default(),

            fn_ty_cache: Default::default(),
        }
    }
}

impl<'i> VisibleVars<'i> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_fn_gen(&mut self, name: &str, f: FunctionGen<'i>) {
        self.function_gens.insert(name.into(), f);
    }

    pub fn push_fn(&mut self, name: &str, sig: &[Type], f: Function<'i>) {
        let mangled = generic_mangle(sig, name);
        self.functions.insert(mangled, f);
    }

    pub fn get_gen_fn(&self, call_site: Span, name: &str) -> Result<&FunctionGen<'i>> {
        if let Some(f) = self.function_gens.get(name) {
            Ok(f)
        } else {
            Err(Error::new_fn_not_found(call_site, name))
        }
    }

    pub fn get_fn(&mut self, call_site: Span, name: &str, sig: &[Type]) -> Result<&Function<'i>> {
        let mangled = generic_mangle(sig, name);
        if let Some(f) = self.functions.get_mut(&mangled) {
            if f.type_checking {
                return Ok(self.functions.get(&mangled).unwrap());
            }
            f.type_checking = true;

            let mut f = f.clone();
            f.type_check(self)?;
            self.functions.insert(mangled.clone(), f);
            Ok(self.functions.get(&mangled).unwrap())
        } else {
            Err(Error::new_fn_not_found(call_site, name))
        }
    }

    pub fn get_fn_ty(&mut self, call_site: Span, name: &str, sig: &[Type]) -> Result<Type> {
        if let Some(&ty) = self.fn_ty_cache.get(name) {
            Ok(ty)
        } else {
            self.fn_ty_cache.insert(name.into(), Type::Unresolved);
            Ok(self.get_fn(call_site, name, sig)?.type_of())
        }
    }

    pub fn push_var(&mut self, name: &str, ty: Type) {
        log::debug!("push {name} = {ty}");
        self.vars
            .last_mut()
            .as_mut()
            .unwrap()
            .insert(name.into(), ty);
    }

    pub fn get_var(&self, name: &str) -> Option<Type> {
        let ty = self
            .vars
            .iter()
            .rev()
            .find_map(|map| map.get(name))
            .cloned()?;

        log::debug!("get {name} = {ty}");

        Some(ty)
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
            format!("unary operator: '{op}' cannot be applied to type: '{ty}'"),
        )
    }

    pub fn new_invalid_binary_op(span: Span, lhs: Type, op: BinaryOp, rhs: Type) -> Self {
        Self::new_spanned(
            span,
            format!("binary operator: '{op}' cannot be applied to lhs: '{lhs}' and rhs: '{rhs}'"),
        )
    }

    pub fn new_type_mismatch(span: Span, expect: &Type, got: &Type) -> Self {
        Self::new_spanned(span, format!("expected type: '{expect}' but got: '{got}'"))
    }

    pub fn new_rule_mismatch(span: Span, expect: Rule, got: Rule) -> Self {
        let bt = Backtrace::new();
        Self::new_spanned(
            span,
            format!("expected rule: '{expect:?}' but got: '{got:?}'. Internal error {bt:?}",),
        )
    }

    pub fn new_var_not_found(span: Span, expect: &str) -> Self {
        Self::new_spanned(
            span,
            format!("variable '{expect}' not found within accessible scopes"),
        )
    }

    pub fn new_fn_not_found(span: Span, expect: &str) -> Self {
        Self::new_spanned(
            span,
            format!("function '{expect}' not found within accessible scopes"),
        )
    }

    pub fn new_argc_mismatch(span: Span, expect: usize, got: usize) -> Self {
        Self::new_spanned(
            span,
            format!("function got {got} arguments but expected {expect}"),
        )
    }

    pub fn new_not_callable(span: Span, expect: &str) -> Self {
        Self::new_spanned(span, format!("variable '{expect}' is not callable"))
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
        Err(Error::new_rule_mismatch(span.clone(), expect, got))
    }
}

// -----------
// Parse trait
// -----------

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
    fn type_check_impl(&mut self, vars: &mut VisibleVars<'i>) -> Result<()>;

    #[deprecated]
    fn type_of_impl(&self) -> Option<Type>;

    fn type_check(&mut self, vars: &mut VisibleVars<'i>) -> Result<()> {
        #[allow(deprecated)]
        let _ = self.type_check_impl(vars)?;

        let _ = self.type_of();

        Ok(())
    }

    fn type_of(&self) -> Type {
        #[allow(deprecated)]
        self.type_of_impl().expect("Wasn't type checked")
    }
}
