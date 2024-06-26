# SaBA

Sample Browser App (SaBA) on single process / thread.

## how to run

For CUI browser on your host computer,
```
$ cargo run --features=cui --bin=saba_cui --no-default-features
```

For GUI browser on your host computer,
```
$ cargo run --features=gui --bin=saba_gui
```

For [Wasabi OS](https://github.com/hikalium/wasabi),

To build,

```
$ cargo build --features=wasabi --bin=saba --target=x86_64-unknown-none
```

You may not be able to run via `cargo run` because the target archtecture
is different from your environment.
So use a helper script to run the code,

```
$ ./run_on_wasabi.sh
```
