//! This is a helper function to construct HTML string from DOM tree.

use crate::renderer::html::dom::{Node, NodeKind};
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn dom_to_html(root: &Option<Rc<RefCell<Node>>>) -> String {
    let mut html = String::new();
    dom_to_html_internal(root, &mut html);
    html
}

fn dom_to_html_internal(node: &Option<Rc<RefCell<Node>>>, html: &mut String) {
    match node {
        Some(n) => {
            // open tag
            match n.borrow().kind() {
                NodeKind::Document => {}
                NodeKind::Element(ref e) => {
                    html.push_str("<");
                    html.push_str(&e.kind().to_string());
                    for attr in e.attributes() {
                        html.push_str(" ");
                        html.push_str(&attr.name());
                        html.push_str("=");
                        html.push_str(&attr.value());
                    }
                    html.push_str(">");
                }
                NodeKind::Text(ref s) => html.push_str(s),
            }

            dom_to_html_internal(&n.borrow().first_child(), html);

            // close tag
            match n.borrow().kind() {
                NodeKind::Document => {}
                NodeKind::Element(ref e) => {
                    html.push_str("</");
                    html.push_str(&e.kind().to_string());
                    html.push_str(">");
                }
                NodeKind::Text(_s) => {}
            }

            dom_to_html_internal(&n.borrow().next_sibling(), html);
        }
        None => return,
    }
}
