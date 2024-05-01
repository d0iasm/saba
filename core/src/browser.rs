use crate::renderer::page::Page;
use alloc::rc::Rc;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Browser {
    // TODO: support multiple tabs/pages. This browser currently supports only one page.
    page: Rc<RefCell<Page>>,
}

impl Browser {
    pub fn new() -> Rc<RefCell<Self>> {
        let page = Rc::new(RefCell::new(Page::new()));

        let browser = Rc::new(RefCell::new(Self { page: page.clone() }));

        page.borrow_mut().set_browser(Rc::downgrade(&browser));

        browser
    }

    // Called when a browser is clicked.
    pub fn clicked(&self, _position: (i64, i64)) {}

    pub fn current_page(&self) -> Rc<RefCell<Page>> {
        self.page.clone()
    }
}
