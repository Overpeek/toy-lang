use super::{err::CompileResult, module::Module};
use inkwell::values::BasicValueEnum;

//

pub use self::access::*;
pub use self::assign::*;
pub use self::binary::*;
pub use self::branch::*;
pub use self::call::*;
pub use self::expr::*;
pub use self::function::*;
pub use self::lit::*;
pub use self::module::*;
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
pub mod lit;
pub mod module;
pub mod scope;
pub mod statement;
pub mod term;
pub mod unary;

//

pub type CodeGenResult<'ctx> = CompileResult<Option<BasicValueEnum<'ctx>>>;

pub trait CodeGen {
    fn code_gen<'ctx>(&self, module: &mut Module<'ctx>) -> CodeGenResult<'ctx>;
}
