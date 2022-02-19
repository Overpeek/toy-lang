use toy_lang::compiler::instance::Compiler;

pub fn main() {
    env_logger::init();

    let compiler = Compiler::new();

    let result: f64 = compiler
        .module_from_source(
            r#"
			fn x() -> f64 {
				5.0
			}

			4.4 * x()
		"#,
        )
        .unwrap()
        .exec()
        .unwrap();

    log::info!("result = {result}");
}
