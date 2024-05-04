use crate::browser::Browser;
use alloc::rc::Weak;
use alloc::string::String;
use core::cell::RefCell;

// disable console features because already borrowed error happens.
// TODO: move logs to global mutable object.
pub fn console_debug(browser: Weak<RefCell<Browser>>, log: String) {
    if let Some(browser) = browser.upgrade() {
        //browser.borrow_mut().console_debug(log);
    }
}

pub fn console_warning(browser: Weak<RefCell<Browser>>, log: String) {
    if let Some(browser) = browser.upgrade() {
        //browser.borrow_mut().console_warning(log);
    }
}

pub fn console_error(browser: Weak<RefCell<Browser>>, log: String) {
    if let Some(browser) = browser.upgrade() {
        //browser.borrow_mut().console_error(log);
    }
}

/*
/// for debug
pub fn print_dom(ui: &Rc<RefCell>, node: &Option<Rc<RefCell<Node>>>, depth: usize) {
    match node {
        Some(n) => {
            ui.borrow_mut()
                .console_debug("  ".repeat(depth).to_string());
            ui.borrow_mut()
                .console_debug(format!("{:?}", n.borrow().kind()));
            print_dom(ui, &n.borrow().first_child(), depth + 1);
            print_dom(ui, &n.borrow().next_sibling(), depth);
        }
        None => (),
    }
}

/// for debug
pub fn print_layout_tree(ui: &Rc<RefCell>, node: &Option<Rc<RefCell<LayoutObject>>>, depth: usize) {
    match node {
        Some(n) => {
            ui.borrow_mut()
                .console_debug("  ".repeat(depth).to_string());
            ui.borrow_mut().console_debug(format!(
                "{:?} {:?}",
                n.borrow().kind(),
                n.borrow().style()
            ));
            print_layout_tree(ui, &n.borrow().first_child(), depth + 1);
            print_layout_tree(ui, &n.borrow().next_sibling(), depth);
        }
        None => (),
    }
}

/// for debug
pub fn print_ast(ui: &Rc<RefCell>, program: &Program) {
    for node in program.body() {
        ui.borrow_mut().console_debug(format!("{:?}", node));
    }
}
*/
