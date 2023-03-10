extern crate alloc;

use alloc::rc::Rc;
use common::error::Error;
use core::cell::RefCell;
use net::http::HttpResponse;
use renderer::page::page::Page;
use renderer::ui::UiObject;

pub struct Browser<U: UiObject> {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    ui: Rc<RefCell<U>>,
    page: Rc<RefCell<Page<U>>>,
}

impl<U: UiObject> Browser<U> {
    pub fn new(ui: Rc<RefCell<U>>) -> Self {
        let page = Rc::new(RefCell::new(Page::new()));
        Self { ui, page }
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

    pub fn ui(&self) -> Rc<RefCell<U>> {
        self.ui.clone()
    }

    pub fn page(&self) -> Rc<RefCell<Page<U>>> {
        self.page.clone()
    }
}
