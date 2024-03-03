use crate::display_item::DisplayItem;
use crate::event::Event;
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
    events: Vec<Event>,
    display_items: Vec<DisplayItem>,
    logs: Vec<Log>,
}

impl Browser {
    pub fn new(page: Rc<RefCell<Page>>) -> Self {
        Self {
            page,
            events: Vec::new(),
            display_items: Vec::new(),
            logs: Vec::new(),
        }
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
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
