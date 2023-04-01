//! UI interface that should be implemented in another module.
use crate::common::error::Error;
use net::http::HttpResponse;

pub trait UiObject {
    fn new() -> Self;
    fn console_debug(&mut self, log: String);
    fn console_warning(&mut self, log: String);
    fn console_error(&mut self, log: String);
    fn start(&mut self, handle_url: fn(String) -> Result<HttpResponse, Error>)
        -> Result<(), Error>;
}
