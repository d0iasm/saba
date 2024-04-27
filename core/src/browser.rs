use crate::display_item::DisplayItem;
use crate::log::{Log, LogLevel};
use crate::renderer::page::Page;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

#[derive(Debug, Clone)]
struct Subresource {
    src: String,
    // TODO: update a type of resource
    resource: u16,
}

impl Subresource {
    fn new(src: String) -> Self {
        Self { src, resource: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Browser {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    page: Rc<RefCell<Page>>,
    display_items: Vec<DisplayItem>,
    logs: Vec<Log>,
    subresources: Vec<Subresource>,
}

impl Browser {
    pub fn new() -> Rc<RefCell<Self>> {
        let page = Rc::new(RefCell::new(Page::new()));

        let browser = Rc::new(RefCell::new(Self {
            page: page.clone(),
            display_items: Vec::new(),
            logs: Vec::new(),
            subresources: Vec::new(),
        }));

        page.borrow_mut().set_browser(Rc::downgrade(&browser));

        browser
    }

    pub fn push_display_item(&mut self, item: DisplayItem) {
        self.display_items.push(item);
    }

    pub fn push_url_for_subresource(&mut self, src: String) {
        self.subresources.push(Subresource::new(src));
    }

    pub fn subresource(&self, src: String) -> u16 {
        for s in &self.subresources {
            if s.src == src {
                return s.resource;
            }
        }
        0
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
