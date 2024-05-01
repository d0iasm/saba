///! This is a helper module for activation behaviors.
///! Activation behaviors are used as a default behavior on certain event targets.
///! e.g., <a> and <area> elements, in response to synthetic MouseEvent events whose type attribute is click.
///! https://dom.spec.whatwg.org/#eventtarget-activation-behavior
use crate::renderer::dom::event::Event;
use crate::renderer::html::dom::ElementKind;
use crate::renderer::html::dom::Node;
use crate::renderer::html::dom::NodeKind;
use alloc::rc::Rc;
use core::cell::RefCell;

///! https://dom.spec.whatwg.org/#eventtarget-activation-behavior
pub type ActivationBehavior = fn(node: Rc<RefCell<Node>>, e: Event);

pub fn get_activation_behavior(node_kind: &NodeKind) -> Option<ActivationBehavior> {
    match node_kind {
        NodeKind::Document | NodeKind::Text(_) => return None,
        NodeKind::Element(e) => {
            match e.kind() {
                ElementKind::A => {
                    // Return an activation behavior if the element is <a> and it has "href" attribute.
                    // https://html.spec.whatwg.org/multipage/links.html#links-created-by-a-and-area-elements
                    if let Some(_href) = e.get_attribute("href") {
                        return Some(follow_hyperlink);
                    }
                    return None;
                }
                _ => return None,
            }
        }
    }
}

/// https://html.spec.whatwg.org/multipage/links.html#links-created-by-a-and-area-elements
/// https://html.spec.whatwg.org/multipage/links.html#following-hyperlinks-2
/// https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate
fn follow_hyperlink(node: Rc<RefCell<Node>>, _event: Event) {
    let element = match node.borrow().get_element() {
        Some(e) => e,
        None => return,
    };

    // "1. If element has no href attribute, then return."
    let _href = match element.get_attribute("href") {
        Some(href) => href,
        None => return,
    };

    // navigate
}
