use crate::display_item::DisplayItem;
use crate::log::{Log, LogLevel};
use crate::renderer::page::Page;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Browser {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    page: Rc<RefCell<Page>>,
    display_items: Vec<DisplayItem>,
    logs: Vec<Log>,
}

impl Browser {
    pub fn new() -> Rc<RefCell<Self>> {
        let page = Rc::new(RefCell::new(Page::new()));

        let browser = Rc::new(RefCell::new(Self {
            page: page.clone(),
            display_items: Vec::new(),
            logs: Vec::new(),
        }));

        page.borrow_mut().set_browser(Rc::downgrade(&browser));

        browser
    }

    pub fn start_navigation(&self) {}

    pub fn push_url_for_subresource(&mut self, src: String) {
        self.page.borrow_mut().push_url_for_subresource(src);
    }

    pub fn push_display_item(&mut self, item: DisplayItem) {
        self.display_items.push(item);
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

    pub fn page(&self) -> Rc<RefCell<Page>> {
        self.page.clone()
    }

    pub fn display_items(&self) -> Vec<DisplayItem> {
        self.display_items.clone()
    }

    pub fn clear_display_items(&mut self) {
        self.display_items = Vec::new();
    }

    pub fn logs(&self) -> Vec<Log> {
        self.logs.clone()
    }

    pub fn clear_logs(&mut self) {
        self.logs = Vec::new();
    }
}
