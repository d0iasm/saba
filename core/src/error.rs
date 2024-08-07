use alloc::string::String;

/// https://doc.rust-lang.org/nightly/std/io/enum.ErrorKind.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Network(String),
    UnexpectedInput(String),
    InvalidUI(String),
    Other(String),
}
