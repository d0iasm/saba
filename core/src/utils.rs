use crate::browser::Browser;
use crate::display_item::DisplayItem;
use crate::renderer::layout::layout_object::LayoutObject;
use alloc::rc::Weak;
use alloc::string::String;
use core::cell::RefCell;

pub fn add_rect_display_item(layout_object: &LayoutObject) {
    let style = layout_object.style();
    let point = layout_object.point();
    let size = layout_object.size();
    let browser = match layout_object.browser().upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().push_display_item(DisplayItem::Rect {
        style,
        layout_point: point,
        layout_size: size,
    });
}

pub fn add_link_display_item(layout_object: &LayoutObject, href: String, link_text: String) {
    let style = layout_object.style();
    let point = layout_object.point();
    let browser = match layout_object.browser().upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().push_display_item(DisplayItem::Link {
        text: link_text,
        destination: href,
        style,
        layout_point: point,
    });
}

pub fn add_text_display_item(layout_object: &LayoutObject, text: String) {
    let style = layout_object.style();
    let point = layout_object.point();
    let browser = match layout_object.browser().upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().push_display_item(DisplayItem::Text {
        text,
        style,
        layout_point: point,
    });
}

pub fn console_debug(browser: Weak<RefCell<Browser>>, log: String) {
    let browser = match browser.upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().console_debug(log);
}

pub fn console_warning(browser: Weak<RefCell<Browser>>, log: String) {
    let browser = match browser.upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().console_warning(log);
}

pub fn console_error(browser: Weak<RefCell<Browser>>, log: String) {
    let browser = match browser.upgrade() {
        Some(browser) => browser,
        None => return,
    };

    browser.borrow_mut().console_error(log);
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
