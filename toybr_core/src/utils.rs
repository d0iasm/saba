use crate::browser::Browser;
use crate::common::ui::UiObject;
use crate::renderer::html::dom::Node;
use crate::renderer::js::ast::Program;
use crate::renderer::layout::layout_object::LayoutObject;
use alloc::rc::{Rc, Weak};
use core::cell::RefCell;

/*
pub fn println<U: UiObject>(browser: Weak<RefCell<Browser<U>>>, text: String) {
    let browser = match browser.upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().println(text);
}
*/

pub fn println<U: UiObject>(layout_object: &LayoutObject<U>, text: String) {
    let style = layout_object.style();
    let position = layout_object.position();
    let browser = match layout_object.browser().upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().println(text, style, position);
}

pub fn console_debug<U: UiObject>(browser: Weak<RefCell<Browser<U>>>, log: String) {
    let browser = match browser.upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().console_debug(log);
}

pub fn console_warning<U: UiObject>(browser: Weak<RefCell<Browser<U>>>, log: String) {
    let browser = match browser.upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().console_warning(log);
}

pub fn console_error<U: UiObject>(browser: Weak<RefCell<Browser<U>>>, log: String) {
    let browser = match browser.upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().console_error(log);
}

/// for debug
pub fn print_dom<U: UiObject>(ui: &Rc<RefCell<U>>, node: &Option<Rc<RefCell<Node>>>, depth: usize) {
    match node {
        Some(n) => {
            ui.borrow_mut()
                .console_debug(format!("{}", "  ".repeat(depth)));
            ui.borrow_mut()
                .console_debug(format!("{:?}", n.borrow().kind()));
            print_dom(ui, &n.borrow().first_child(), depth + 1);
            print_dom(ui, &n.borrow().next_sibling(), depth);
        }
        None => return,
    }
}

/// for debug
pub fn print_layout_tree<U: UiObject>(
    ui: &Rc<RefCell<U>>,
    node: &Option<Rc<RefCell<LayoutObject<U>>>>,
    depth: usize,
) {
    match node {
        Some(n) => {
            ui.borrow_mut()
                .console_debug(format!("{}", "  ".repeat(depth)));
            ui.borrow_mut().console_debug(format!(
                "{:?} {:?}",
                n.borrow().kind(),
                n.borrow().style
            ));
            print_layout_tree(ui, &n.borrow().first_child(), depth + 1);
            print_layout_tree(ui, &n.borrow().next_sibling(), depth);
        }
        None => return,
    }
}

/// for debug
pub fn print_ast<U: UiObject>(ui: &Rc<RefCell<U>>, program: &Program) {
    for node in program.body() {
        ui.borrow_mut().console_debug(format!("{:?}", node));
    }
}
