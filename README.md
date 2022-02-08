<div align="center">

# toy-lang

WIP simple script lang for games

[![dependency status](https://deps.rs/repo/github/Overpeek/toy-lang/status.svg)](https://deps.rs/repo/github/Overpeek/toy-lang)
[![build status](https://github.com/Overpeek/toy-lang/actions/workflows/rust.yml/badge.svg)](https://github.com/Overpeek/toy-lang/actions)
 
 </div>

### Goals:
 - rust-like
 - static typing
 - jit compilation with llvm
 - easy to use
 - type interfacing (between rust and script code)

### Type interfacing goal:
```rust
#[derive(ObjInterface)]
pub struct Obj {
    val: u32
}

let result: Obj = run_code(r#"
    let obj: Obj = Obj { val: 3 };
    obj
"#).unwrap();
```
