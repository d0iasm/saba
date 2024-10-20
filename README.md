# SaBA (Sample Brorwser Application)

Sample Browser Application (SaBA) on a single thread/process. The browser runs on Mac/Linux as a CUI application or on [Wasabi OS](https://github.com/hikalium/wasabi) as a GUI application.

The book is published on Nov 9, 2024!

[『［作って学ぶ］ブラウザのしくみ──HTTP、HTML、CSS、JavaScriptの裏側』](https://amzn.asia/d/j1XxxsN) is written in Japanese. The book describes how to make your own browser from scratch in Rust. The code used in the book is based on this repository.

<img src="https://m.media-amazon.com/images/I/81fO73On7jL._SL1500_.jpg" alt="［作って学ぶ］ブラウザのしくみ──HTTP、HTML、CSS、JavaScriptの裏側" width="300"/>

## Supported Features

It supports sending/receiving HTTP GET request/response, basic HTML tags, basic CSS syntax and basic JavaScript syntax.

- [x] Handle HTTP GET request/response
- [x] Basic HTML tags (<html>, <head> ,<body>, <h1>, <h2>, <p>, etc.)
- [x] Basic CSS syntax ("background-color", "color", "display", etc.)
- [x] Basic JavaScript syntax (addition, subtraction, variable, function)

Upcoming features:

- [ ] QUIC (HTTP/3) protocol
- [ ] HTTPS
- [ ] More HTML tags
- [ ] More CSS syntax
- [ ] GUI on Mac/Linux

## How to Run

### GUI Application on Wasabi OS

You may not be able to run via `cargo run` because the target archtecture
is different from your environment.
So use a helper script to run the code,

```
$ ./run_on_wasabi.sh
```

### CUI Application on Mac/Linux

For CUI browser on your host computer,

```
$ cargo run --features=cui --bin=saba_cui --no-default-features
```

### GUI Application on Mac/Linux

GUI on Mac/Linux is not supported yet.

For GUI browser on your host computer,

```
$ cargo run --features=gui --bin=saba_gui --no-default-features
```