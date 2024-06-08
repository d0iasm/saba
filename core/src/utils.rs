use crate::browser::Browser;
use crate::renderer::dom::node::Node;
use crate::renderer::js::ast::Program;
use crate::renderer::layout::layout_object::LayoutObject;
use alloc::format;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
use core::cell::RefCell;

pub fn console_debug(browser: &Weak<RefCell<Browser>>, log: String) {
    if let Some(browser) = browser.upgrade() {
        browser.borrow_mut().console_debug(log);
    }
}

pub fn console_warning(browser: &Weak<RefCell<Browser>>, log: String) {
    if let Some(browser) = browser.upgrade() {
        browser.borrow_mut().console_warning(log);
    }
}

pub fn console_error(browser: &Weak<RefCell<Browser>>, log: String) {
    if let Some(browser) = browser.upgrade() {
        browser.borrow_mut().console_error(log);
    }
}

/// for debug
pub fn convert_dom_to_string(root: &Option<Rc<RefCell<Node>>>) -> String {
    let mut result = String::new();
    convert_dom_to_string_internal(root, 0, &mut result);
    result
}

fn convert_dom_to_string_internal(
    node: &Option<Rc<RefCell<Node>>>,
    depth: usize,
    result: &mut String,
) {
    match node {
        Some(n) => {
            result.push_str(&"  ".repeat(depth));
            result.push_str(&format!("{:?}", n.borrow().kind()));
            convert_dom_to_string_internal(&n.borrow().first_child(), depth + 1, result);
            convert_dom_to_string_internal(&n.borrow().next_sibling(), depth, result);
        }
        None => return,
    }
}

/// for debug
pub fn convert_layout_tree_to_string(node: &Option<Rc<RefCell<LayoutObject>>>) -> String {
    let mut result = String::new();
    convert_layout_tree_to_string_internal(node, 0, &mut result);
    result
}

fn convert_layout_tree_to_string_internal(
    node: &Option<Rc<RefCell<LayoutObject>>>,
    depth: usize,
    result: &mut String,
) {
    match node {
        Some(n) => {
            result.push_str(&"  ".repeat(depth));
            result.push_str(&format!("{:?} {:?}", n.borrow().kind(), n.borrow().style()));
            convert_layout_tree_to_string_internal(&n.borrow().first_child(), depth + 1, result);
            convert_layout_tree_to_string_internal(&n.borrow().next_sibling(), depth, result);
        }
        None => return,
    }
}

/// for debug
pub fn convert_ast_to_string(program: &Program) -> String {
    let mut result = String::new();
    for node in program.body() {
        result.push_str(&format!("{:?}", node));
    }
    result
}
