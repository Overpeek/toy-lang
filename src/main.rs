use colorful::Colorful;
use compiler::Compiler;
use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

use crate::compiler::Optimize;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod compiler;

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

#[allow(unused)]
fn benchmark(opt: Optimize) {
    println!("Benchmarking ...");
    let compiler = Compiler::default();

    // compiler
    let (c_setup, c_runs) = bench(
        || {
            let ast = ast::parse_file("tests/script.tls").unwrap();
            compiler.module(&ast, opt)
        },
        |module| module.exec(),
        -46845.0,
    );

    // zero-cost
    let (z_setup, z_runs) = bench(|| {}, |_| {}, ());

    println!(
        "Compiler setup took:    {}",
        format!("{:>8}", format!("{:.1?}", c_setup))
            .green()
            .to_string()
            .as_str()
    );
    println!(
        "Zero-cost setup took:   {}",
        format!("{:>8}", format!("{:.1?}", z_setup))
            .green()
            .to_string()
            .as_str()
    );
    println!(
        "Compiled ran:    {} times in 3 sec",
        format!("{:>15}", c_runs).yellow(),
    );
    println!(
        "Zero-cost ran:   {} times in 3 sec ({}× faster)",
        format!("{:>15}", z_runs).yellow(),
        format!("{:.1}", z_runs as f64 / c_runs as f64).red()
    );
}

#[allow(unused)]
fn run(opt: Optimize) {
    println!("Running ...");
    let ast = ast::parse_file("tests/script.tls").unwrap();
    let compiler = Compiler::default();
    let mut module = compiler.module(&ast, opt);

    let result = module.exec();

    println!("Result: {}", result);
}

fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();

    let mut opt = Optimize::O2;

    for arg in args {
        let arg = arg.as_str();
        if let Some((_, rhs)) = arg.split_once("-O") {
            opt = match rhs {
                "0" => Optimize::O0,
                "1" => Optimize::O1,
                "2" => Optimize::O2,
                "3" => Optimize::O3,
                other => panic!("Opt level {} does not exist", other),
            }
        }
    }

    // benchmark();
    run(opt);
}
