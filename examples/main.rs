use toy_lang::compiler::instance::Compiler;

pub fn main() {
    env_logger::init();

    let compiler = Compiler::new();

    let result: f64 = compiler
        .module_from_source(
            r#"
                if false {
                    5.0
                } else {
                    if true {
                        2.0
                    } else {
                        1.0
                    }
                }
		    "#,
        )
        .unwrap()
        .exec()
        .unwrap();

    log::info!("result = {result}");
}
