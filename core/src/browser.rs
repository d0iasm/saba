//! The main browser struct to manage pages.

use crate::log::Log;
use crate::log::LogLevel;
use crate::renderer::page::Page;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Browser {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    active_page_index: usize,
    page: Vec<Rc<RefCell<Page>>>,
    logs: Vec<Log>,
}

impl Browser {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut page = Page::new();

        let browser = Rc::new(RefCell::new(Self {
            active_page_index: 0,
            page: Vec::new(),
            logs: Vec::new(),
        }));

        page.set_browser(Rc::downgrade(&browser));
        browser.borrow_mut().page.push(Rc::new(RefCell::new(page)));

        browser
    }

    pub fn current_page(&self) -> Rc<RefCell<Page>> {
        self.page[self.active_page_index].clone()
    }

    pub fn push_url_for_subresource(&mut self, src: String) {
        self.page[self.active_page_index]
            .borrow_mut()
            .push_url_for_subresource(src);
    }

    pub fn logs(&self) -> Vec<Log> {
        self.logs.clone()
    }

    pub fn clear_logs(&mut self) {
        self.logs = Vec::new();
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
}
