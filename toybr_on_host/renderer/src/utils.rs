use crate::html::dom::Node;
use crate::js::ast::Program;
use crate::layout::layout_object::LayoutObject;
use crate::ui::UiObject;
use alloc::rc::Rc;
use core::cell::RefCell;

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
    node: &Option<Rc<RefCell<LayoutObject>>>,
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
