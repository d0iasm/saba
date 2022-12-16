# toybr
A toy Web browser on single process / thread

## how to run

For Linux,

```
$ cargo run -vv --features std --bin toybr
$ cargo run -vv --features std --bin toybr --target aarch64-apple-darwin -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem
```

For [Wasabi OS](https://github.com/hikalium/wasabi),

```
$ cargo run -vv --features nostd --bin toybr_nostd --target x86_64-unknown-elf.json -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem
```

```
$ cargo run --target x86_64-unknown-elf.json
```


For Mac,

```
$ cargo run --target aarch64-apple-darwin
```
