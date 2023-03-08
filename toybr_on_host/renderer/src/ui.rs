//! UI interface that should be implemented in another module.

pub trait UiObject {
    fn new() -> Self;
    fn println(&mut self, text: String);
    fn console_debug(&mut self, log: String);
    fn console_error(&mut self, log: String);
}
