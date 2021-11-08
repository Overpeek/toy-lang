use ast::Ast;
use colorful::Colorful;
use compiler::Compiler;
use interpreter::Memory;
use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

use crate::interpreter::Interpreter;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod compiler;
pub mod interpreter;

fn bench<T, U: Debug + PartialEq, F1: Fn() -> T, F2: Fn(&mut T) -> U>(
    setup: F1,
    bench: F2,
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

fn main() {
    env_logger::init();
    println!("Running ...");
    let file = std::fs::read_to_string("tests/script.tls").unwrap();
    let interpreter = Interpreter::new();
    let compiler = Compiler::default();

    // interpreter
    let (i_setup, i_runs) = bench(
        || {
            let memory = Memory::default();
            let ast = Ast::new(&file).unwrap();
            (memory, ast)
        },
        |(memory, ast)| interpreter.exec(memory, ast),
        -46845.0,
    );

    // compiler
    let (c_setup, c_runs) = bench(
        || {
            let mut module = compiler.module();
            let ast = Ast::new(&file).unwrap();
            module.compile(&ast);
            module
        },
        |module| module.exec(),
        -46845.0,
    );

    // zero-cost
    let (z_setup, z_runs) = bench(|| {}, |_| {}, ());

    println!(
        "Interpreter setup took: {}",
        format!("{:>8}", format!("{:.1?}", i_setup))
            .green()
            .to_string()
            .as_str()
    );
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
        "Interpreted ran: {} times in 3 sec",
        format!("{:>15}", i_runs).yellow()
    );
    println!(
        "Compiled ran:    {} times in 3 sec ({}× faster)",
        format!("{:>15}", c_runs).yellow(),
        format!("{:.1}", c_runs as f64 / i_runs as f64).red()
    );
    println!(
        "Zero-cost ran:   {} times in 3 sec ({}× faster)",
        format!("{:>15}", z_runs).yellow(),
        format!("{:.1}", z_runs as f64 / c_runs as f64).red()
    );
}
