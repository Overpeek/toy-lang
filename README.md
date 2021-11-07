# toy-lang

WIP simple script lang for games

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
