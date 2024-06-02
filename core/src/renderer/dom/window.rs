//! This is a DOM Window object.
//! https://html.spec.whatwg.org/multipage/nav-history-apis.html#window

use crate::browser::Browser;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::page::Page;
use alloc::rc::Rc;
use alloc::rc::Weak;
use core::cell::RefCell;

/// https://html.spec.whatwg.org/multipage/nav-history-apis.html#window
///
/// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/frame/dom_window.h
#[derive(Debug, Clone)]
pub struct Window {
    _browser: Weak<RefCell<Browser>>,
    _page: Weak<RefCell<Page>>,
    document: Rc<RefCell<Node>>,
}

impl Window {
    pub fn new(browser: Weak<RefCell<Browser>>) -> Self {
        let window = Self {
            _browser: browser,
            _page: Weak::new(),
            document: Rc::new(RefCell::new(Node::new(NodeKind::Document))),
        };

        window
            .document
            .borrow_mut()
            .set_window(Rc::downgrade(&Rc::new(RefCell::new(window.clone()))));

        window
    }

    pub fn document(&self) -> Rc<RefCell<Node>> {
        self.document.clone()
    }
}
