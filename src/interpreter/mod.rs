use crate::artefact::ast::Node;

pub fn run_interpreter(ast: &Box<dyn Node>) -> f64 {
    ast.visit()
}
