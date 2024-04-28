///! This is a helper module for activation behaviors.
///! Activation behaviors are used as a default behavior on certain event targets.
///! e.g., <a> and <area> elements, in response to synthetic MouseEvent events whose type attribute is click.
///! https://dom.spec.whatwg.org/#eventtarget-activation-behavior
use crate::renderer::dom::event::Event;
use crate::renderer::html::dom::Element;
use crate::renderer::html::dom::ElementKind;

///! https://dom.spec.whatwg.org/#eventtarget-activation-behavior
pub type ActivationBehavior = fn(element: Element, e: Event);

pub fn get_activation_behavior(element: &Element) -> Option<ActivationBehavior> {
    match element.kind() {
        ElementKind::A => {
            // Return an activation behavior if the element is <a> and it has "href" attribute.
            if let Some(_href) = element.get_attribute("href") {
                return Some(follow_hyperlink);
            }
            return None;
        }
        _ => None,
    }
}

/// https://html.spec.whatwg.org/multipage/links.html#links-created-by-a-and-area-elements
/// https://html.spec.whatwg.org/multipage/links.html#following-hyperlinks-2
/// https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate
fn follow_hyperlink(element: Element, event: Event) {
    // "1. If element has no href attribute, then return."
    let href = match element.get_attribute("href") {
        Some(href) => href,
        None => return,
    };

    // navigate to the href.
}
