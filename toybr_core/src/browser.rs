extern crate alloc;

use crate::common::error::Error;
use crate::common::log::{Log, LogLevel};
use crate::common::ui::UiObject;
use crate::renderer::page::Page;
use alloc::rc::Rc;
use core::cell::RefCell;
use net::http::HttpResponse;

pub struct Browser<U: UiObject> {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    ui: Rc<RefCell<U>>,
    page: Rc<RefCell<Page<U>>>,
    contents: Vec<String>,
    logs: Vec<Log>,
}

impl<U: UiObject> Browser<U> {
    pub fn new(ui: Rc<RefCell<U>>, page: Rc<RefCell<Page<U>>>) -> Self {
        Self {
            ui,
            page,
            contents: Vec::new(),
            logs: Vec::new(),
        }
    }

    pub fn start(&mut self, handle_url: fn(String) -> Result<HttpResponse, Error>) {
        match self.ui.borrow_mut().start(handle_url) {
            Ok(_) => {}
            Err(e) => {
                self.ui
                    .borrow_mut()
                    .console_error(format!("browser is terminated by {:?}", e));
            }
        }
    }

    pub fn println(&mut self, text: String) {
        self.contents.push(text);
    }

    pub fn console_debug(&mut self, log: String) {
        self.logs.push(Log::new(LogLevel::Debug, log));
    }

    pub fn console_warning(&mut self, log: String) {
        self.logs.push(Log::new(LogLevel::Warning, log));
    }

    pub fn console_error(&mut self, log: String) {
        self.logs.push(Log::new(LogLevel::Error, log));
    }

    pub fn ui(&self) -> Rc<RefCell<U>> {
        self.ui.clone()
    }

    pub fn page(&self) -> Rc<RefCell<Page<U>>> {
        self.page.clone()
    }

    pub fn contents(&self) -> Vec<String> {
        self.contents.clone()
    }

    pub fn clear_contents(&mut self) {
        self.contents = Vec::new();
    }

    pub fn logs(&self) -> Vec<Log> {
        self.logs.clone()
    }

    pub fn clear_logs(&mut self) {
        self.logs = Vec::new();
    }
}
