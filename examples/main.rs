use colorful::Colorful;
use std::{
    fmt::Debug,
    time::{Duration, Instant},
};
use toy_lang::compiler::{instance::Compiler, optimizer::OptLevel};

//

const SCRIPT_PATH: &str = "examples/main.tls";

//

fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();

    let mut opt = OptLevel::O2;

    for arg in args {
        let arg = arg.as_str();
        if let Some((_, rhs)) = arg.split_once("-O") {
            opt = match rhs {
                "0" => OptLevel::O0,
                "1" => OptLevel::O1,
                "2" => OptLevel::O2,
                "3" => OptLevel::O3,
                other => panic!("Opt level {} does not exist", other),
            }
        }
    }

    benchmark(opt);
    // run(opt);
}

#[allow(unused)]
fn benchmark(opt: OptLevel) {
    println!("Benchmarking ...");
    let compiler = Compiler::new().with_opt(opt);

    let (c_setup, c_runs) = bench(
        || compiler.module_from_path(SCRIPT_PATH).unwrap(),
        |module| module.exec().unwrap(),
        987,
    );

    println!(
        "Compiling took:    {}",
        format!("{:>8}", format!("{:.1?}", c_setup))
            .green()
            .to_string()
            .as_str()
    );
    println!(
        "Compiled ran:    {} times in 3 sec",
        format!("{:>13}", c_runs).yellow(),
    );
}

#[allow(unused)]
fn run(opt: OptLevel) {
    println!("Running ...");

    let compiler = Compiler::default();
    let mut module = compiler.module_from_path(SCRIPT_PATH).unwrap();
    let result = module.exec();

    println!("Result: {}", result.unwrap());
}

fn bench<T, U: Debug + PartialEq, F1: FnMut() -> T, F2: FnMut(&mut T) -> U>(
    mut setup: F1,
    mut bench: F2,
    expected: U,
) -> (Duration, usize) {
    let instant = Instant::now();
    let mut result = setup();
    let duration = instant.elapsed();

    let instant = Instant::now();
    let mut c_runs = 0;
    while instant.elapsed() < Duration::from_secs(3) {
        c_runs += 1;
        let result = bench(&mut result);
        assert!(
            result == expected,
            "got: {:?}, expected: {:?}",
            result,
            expected
        );
    }
    (duration, c_runs)
}
