# toybr
A toy Web browser on single process / thread

## how to run

For Linux,

```
$ cargo run -vv --features std --bin toybr
```

For [Wasabi OS](https://github.com/hikalium/wasabi),

```
$ cargo run -vv --features nostd --bin toybr_nostd --target x86_64-unknown-elf.json -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem
```
