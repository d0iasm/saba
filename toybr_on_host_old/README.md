# toybr
A toy Web browser on single process / thread

```
$ docker run -v /Users/asami/src/x86test:/x86test --rm -it hikalium/wasabi-builder:latest
```

# How to set up GitHub Codespaces

```
# install Rust
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# set up nightly Rust compiler
$ rustup default nightly

$ rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
```

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