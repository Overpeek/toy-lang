use rand::{distributions::Alphanumeric, Rng};
use toy_lang::{compiler::instance::Compiler, run_code};

#[test]
fn fuzz() {
    let compiler = Compiler::new();
    let mut rng = rand::thread_rng();

    // test for panics
    for _ in 0..2000 {
        let len: usize = rng.gen_range(0..500);
        let buf: String = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect();

        let _ = compiler.module_from_source(buf.as_ref());
    }
}

#[test]
fn fuzz_2() {
    let mut rng = rand::thread_rng();

    // test for panics
    for _ in 0..2000 {
        let len: usize = rng.gen_range(0..500);
        let buf: String = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect();

        let _ = run_code(buf.as_ref());
    }
}
