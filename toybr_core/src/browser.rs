extern crate alloc;

use crate::common::error::Error;
use crate::common::ui::UiObject;
use crate::renderer::page::Page;
use alloc::rc::Rc;
use core::cell::RefCell;
use net::http::HttpResponse;

pub struct Browser<U: UiObject> {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    ui: Rc<RefCell<U>>,
    page: Rc<RefCell<Page<U>>>,
}

impl<U: UiObject> Browser<U> {
    pub fn new(ui: Rc<RefCell<U>>) -> Self {
        let page = Rc::new(RefCell::new(Page::new()));
        page.borrow_mut().set_ui_object(ui.clone());

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
