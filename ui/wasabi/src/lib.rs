#![no_std]

extern crate alloc;

use alloc::rc::Weak;
use alloc::string::String;
use core::cell::RefCell;
use toybr_core::{browser::Browser, error::Error, http::HttpResponse, ui::UiObject};

#[derive(Clone, Debug)]
pub struct WasabiUI {
    browser: Weak<RefCell<Browser<Self>>>,
    input_url: String,
}

impl UiObject for WasabiUI {
    fn new() -> Self {
        Self {
            browser: Weak::new(),
            input_url: String::new(),
        }
    }

    fn console_debug(&mut self, log: String) {}
    fn console_warning(&mut self, log: String) {}
    fn console_error(&mut self, log: String) {}
    fn start(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
