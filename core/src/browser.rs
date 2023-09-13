use crate::display_item::DisplayItem;
use crate::error::Error;
use crate::event::Event;
use crate::http::HttpResponse;
use crate::log::{Log, LogLevel};
use crate::renderer::page::Page;
use crate::ui::UiObject;
use alloc::rc::Rc;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Browser<U: UiObject> {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    ui: Rc<RefCell<U>>,
    page: Rc<RefCell<Page<U>>>,
    events: Vec<Event>,
    display_items: Vec<DisplayItem>,
    logs: Vec<Log>,
}

impl<U: UiObject> Browser<U> {
    pub fn new(ui: Rc<RefCell<U>>, page: Rc<RefCell<Page<U>>>) -> Self {
        Self {
            ui,
            page,
            events: Vec::new(),
            display_items: Vec::new(),
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

    pub fn ui(&self) -> Rc<RefCell<U>> {
        self.ui.clone()
    }

    pub fn page(&self) -> Rc<RefCell<Page<U>>> {
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
