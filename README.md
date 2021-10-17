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

### Working example
```
let value = 9 > 3;

8 * if value {
    8
} else {
    0
}
```
outputs
```
64
```

### TODO:
	jit with llvm
	type interface with proc macro derive
