//! The main browser struct to manage pages.

use crate::http::HttpResponse;
use crate::renderer::page::Page;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Browser {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    active_page_index: usize,
    page: Vec<Page>,
}

impl Browser {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut page = Page::new();

        let browser = Rc::new(RefCell::new(Self {
            active_page_index: 0,
            page: Vec::new(),
        }));

        page.set_browser(Rc::downgrade(&browser));
        browser.borrow_mut().page.push(page);

        browser
    }

    // Called when a browser is clicked.
    pub fn clicked(&self, _position: (i64, i64)) {}

    pub fn receive_response(&mut self, response: HttpResponse) {
        self.page[self.active_page_index].receive_response(response);
    }

    pub fn push_url_for_subresource(&mut self, src: String) {
        self.page[self.active_page_index].push_url_for_subresource(src);
    }

    pub fn clear_display_items(&mut self) {
        self.page[self.active_page_index].clear_display_items();
    }

    pub fn clear_logs(&mut self) {
        self.page[self.active_page_index].clear_logs();
    }

    pub fn console_debug(&mut self, log: String) {
        self.page[self.active_page_index].console_debug(log);
    }

    pub fn console_warning(&mut self, log: String) {
        self.page[self.active_page_index].console_warning(log);
    }

    pub fn console_error(&mut self, log: String) {
        self.page[self.active_page_index].console_error(log);
    }

    pub fn current_page(&self) -> &Page {
        &self.page[self.active_page_index]
    }
}
