//! https://www.w3.org/TR/css-box-3/
//! https://www.w3.org/TR/css-layout-api-1/
//! https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_view.h

use crate::browser::Browser;
use crate::constants::CONTENT_AREA_WIDTH;
use crate::constants::TOOLBAR_HEIGHT;
use crate::constants::WINDOW_PADDING;
use crate::display_item::DisplayItem;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::api::get_target_element_node;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::layout::layout_object::create_layout_object;
use crate::renderer::layout::layout_object::LayoutObject;
use crate::renderer::layout::layout_object::LayoutObjectKind;
use crate::renderer::layout::layout_point::LayoutPoint;
use crate::renderer::layout::layout_size::LayoutSize;
use alloc::rc::{Rc, Weak};
use alloc::vec::Vec;
use core::cell::RefCell;

/// Converts DOM tree to render tree.
fn build_layout_tree(
    browser: Weak<RefCell<Browser>>,
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    // Try to create a LayoutObject. If `display:none`, `layout_object` is None.
    let mut target_node = node.clone();
    let mut layout_object = create_layout_object(browser.clone(), node, parent_obj, cssom);
    // If `layout_object` is None, try to create a LayoutObject with the next sibling.
    while layout_object.is_none() {
        if let Some(n) = target_node {
            target_node = n.borrow().next_sibling().clone();
            layout_object = create_layout_object(browser.clone(), &target_node, parent_obj, cssom);
        } else {
            // Return here because a DOM node doesn't exist (= the end of DOM tree).
            return layout_object;
        }
    }

    if let Some(n) = target_node {
        let original_first_child = n.borrow().first_child();
        let original_next_sibling = n.borrow().next_sibling();
        let mut first_child = build_layout_tree(
            browser.clone(),
            &original_first_child,
            &layout_object,
            cssom,
        );
        let mut next_sibling =
            build_layout_tree(browser.clone(), &original_next_sibling, &None, cssom);

        // if the original first child node is "display:none" and the original first child
        // node has a next sibiling node, treat the next sibling node as a new first child
        // node.
        if first_child.is_none() && original_first_child.is_some() {
            let mut original_dom_node = original_first_child
                .expect("first child should exist")
                .borrow()
                .next_sibling();

            loop {
                first_child =
                    build_layout_tree(browser.clone(), &original_dom_node, &layout_object, cssom);

                // check the next sibling node
                if first_child.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }

                break;
            }
        }

        // if the original next sibling node is "display:none" and the original next
        // sibling node has a next sibling node, treat the next sibling node as a new next
        // sibling node.
        if next_sibling.is_none() && n.borrow().next_sibling().is_some() {
            let mut original_dom_node = original_next_sibling
                .expect("first child should exist")
                .borrow()
                .next_sibling();

            loop {
                next_sibling = build_layout_tree(browser.clone(), &original_dom_node, &None, cssom);

                if next_sibling.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }

                break;
            }
        }

        let obj = match layout_object {
            Some(ref obj) => obj,
            None => panic!("render object should exist here"),
        };
        obj.borrow_mut().set_first_child(first_child);
        obj.borrow_mut().set_next_sibling(next_sibling);
    }

    layout_object
}

/// LayoutView is the root of the layout tree.
/// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_view.h;drc=0e9a0b6e9bb6ec59521977eec805f5d0bca833e0;bpv=1;bpt=1;l=64
#[derive(Debug, Clone)]
pub struct LayoutView {
    root: Option<Rc<RefCell<LayoutObject>>>,
}

impl LayoutView {
    pub fn new(
        browser: Weak<RefCell<Browser>>,
        root: Rc<RefCell<Node>>,
        cssom: &StyleSheet,
    ) -> Self {
        // A layout object should be created for a flow content.
        // https://html.spec.whatwg.org/multipage/dom.html#flow-content-2
        let body_root = get_target_element_node(Some(root), ElementKind::Body);

        let mut tree = Self {
            root: build_layout_tree(browser, &body_root, &None, cssom),
        };

        tree.update_layout();

        tree
    }

    fn calculate_node_size(node: &Option<Rc<RefCell<LayoutObject>>>, parent_size: LayoutSize) {
        match node {
            Some(n) => {
                // For block elements, we should layout the size before calling children.
                if n.borrow().kind() == LayoutObjectKind::Block {
                    n.borrow_mut().compute_size(parent_size);
                }

                let first_child = n.borrow().first_child();
                Self::calculate_node_size(&first_child, n.borrow().size());

                let next_sibling = n.borrow().next_sibling();
                Self::calculate_node_size(&next_sibling, parent_size);

                // TODO: optimize this code because we call compute_size() twice.
                // For inline, text elements and the height of block elements, we should layout the size after calling children.
                n.borrow_mut().compute_size(parent_size);
            }
            None => (),
        }
    }

    fn calculate_node_position(
        node: &Option<Rc<RefCell<LayoutObject>>>,
        parent_point: LayoutPoint,
        previous_sibiling_kind: LayoutObjectKind,
        previous_sibiling_point: Option<LayoutPoint>,
        previous_sibiling_size: Option<LayoutSize>,
    ) {
        match node {
            Some(n) => {
                n.borrow_mut().compute_position(
                    parent_point,
                    previous_sibiling_kind,
                    previous_sibiling_point,
                    previous_sibiling_size,
                );

                let first_child = n.borrow().first_child();
                Self::calculate_node_position(
                    &first_child,
                    n.borrow().point(),
                    previous_sibiling_kind,
                    previous_sibiling_point,
                    previous_sibiling_size,
                );

                let next_sibling = n.borrow().next_sibling();
                Self::calculate_node_position(
                    &next_sibling,
                    parent_point,
                    n.borrow().kind(),
                    Some(n.borrow().point()),
                    Some(n.borrow().size()),
                );
            }
            None => (),
        }
    }

    /// Calculate the layout point.
    fn update_layout(&mut self) {
        Self::calculate_node_size(&self.root, LayoutSize::new(CONTENT_AREA_WIDTH, 0));

        Self::calculate_node_position(
            &self.root,
            LayoutPoint::new(WINDOW_PADDING, TOOLBAR_HEIGHT + WINDOW_PADDING),
            LayoutObjectKind::Block,
            None,
            None,
        );
    }

    pub fn root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.root.clone()
    }

    fn paint_node(node: &Option<Rc<RefCell<LayoutObject>>>, display_items: &mut Vec<DisplayItem>) {
        match node {
            Some(n) => {
                if let Some(item) = n.borrow_mut().paint() {
                    display_items.push(item);
                }

                let first_child = n.borrow().first_child();
                Self::paint_node(&first_child, display_items);

                let next_sibling = n.borrow().next_sibling();
                Self::paint_node(&next_sibling, display_items);
            }
            None => (),
        }
    }

    /// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_view.h;drc=0e9a0b6e9bb6ec59521977eec805f5d0bca833e0;bpv=1;bpt=1;l=155
    pub fn paint(&self) -> Vec<DisplayItem> {
        let mut display_items = Vec::new();

        Self::paint_node(&self.root, &mut display_items);

        display_items
    }

    fn find_node_by_position_internal(
        node: &Option<Rc<RefCell<LayoutObject>>>,
        position: (i64, i64),
    ) -> Option<Rc<RefCell<LayoutObject>>> {
        match node {
            Some(n) => {
                // currently, position is currectly calculated only for block elements.
                if n.borrow().kind() != LayoutObjectKind::Block {
                    return None;
                }

                let first_child = n.borrow().first_child();
                let result1 = Self::find_node_by_position_internal(&first_child, position);
                if result1.is_some() {
                    return result1;
                }

                let next_sibling = n.borrow().next_sibling();
                let result2 = Self::find_node_by_position_internal(&next_sibling, position);
                if result2.is_some() {
                    return result2;
                }

                if n.borrow().point().x() <= position.0
                    && position.0 <= (n.borrow().point().x() + n.borrow().size().width())
                    && n.borrow().point().y() <= position.1
                    && position.1 <= (n.borrow().point().y() + n.borrow().size().height())
                {
                    return Some(n.clone());
                }
                None
            }
            None => None,
        }
    }

    /// Returns a LayoutObject placed on `position`. None if it doesn't exist.
    pub fn find_node_by_position(&self, position: (i64, i64)) -> Option<Rc<RefCell<LayoutObject>>> {
        Self::find_node_by_position_internal(&self.root(), position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;
    use crate::renderer::css::cssom::CssParser;
    use crate::renderer::css::token::CssTokenizer;
    use crate::renderer::dom::api::get_style_content;
    use crate::renderer::dom::node::Element;
    use crate::renderer::dom::node::NodeKind;
    use crate::renderer::html::parser::HtmlParser;
    use crate::renderer::html::token::HtmlTokenizer;
    use alloc::string::String;

    fn create_layout_view(html: String) -> LayoutView {
        let browser = Browser::new();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(Rc::downgrade(&browser), t).construct_tree();
        let dom = window.borrow().document();
        let style = get_style_content(dom.clone());
        let css_tokenizer = CssTokenizer::new(style);
        let cssom = CssParser::new(Rc::downgrade(&browser), css_tokenizer).parse_stylesheet();
        LayoutView::new(Rc::downgrade(&browser), dom, &cssom)
    }

    #[test]
    fn test_empty() {
        let layout_view = create_layout_view("".to_string());
        assert_eq!(None, layout_view.root());
    }

    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>".to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone()
                .expect("root should exist")
                .borrow()
                .node_kind()
        );
    }

    #[test]
    fn test_block() {
        let html = "<html><head></head><body><p></p></body></html>".to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone()
                .expect("root should exist")
                .borrow()
                .node_kind()
        );

        let p = root.expect("root should exist").borrow().first_child();
        assert!(p.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            p.clone().expect("p node should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("p", Vec::new())),
            p.clone().expect("p node should exist").borrow().node_kind()
        );
    }

    #[test]
    fn test_text() {
        let html = "<html><head></head><body>text</body></html>".to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone()
                .expect("root should exist")
                .borrow()
                .node_kind()
        );

        let text = root.expect("root should exist").borrow().first_child();
        assert!(text.is_some());
        assert_eq!(
            LayoutObjectKind::Text,
            text.clone()
                .expect("text node should exist")
                .borrow()
                .kind()
        );
        assert_eq!(
            NodeKind::Text("text".to_string()),
            text.clone()
                .expect("text node should exist")
                .borrow()
                .node_kind()
        );
    }

    #[test]
    fn test_display_none() {
        let html = "<html><head><style>body{display:none;}</style></head><body>text</body></html>"
            .to_string();
        let layout_view = create_layout_view(html);

        assert_eq!(None, layout_view.root());
    }

    #[test]
    fn test_hidden_class() {
        let html = r#"<html>
<head>
<style>
  .hidden {
    display: none;
  }
</style>
</head>
<body>
  <a class="hidden">link1</a>
  <p></p>
  <p class="hidden"><a>link2</a></p>
</body>
</html>"#
            .to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone()
                .expect("root should exist")
                .borrow()
                .node_kind()
        );

        let p = root.expect("root should exist").borrow().first_child();
        assert!(p.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            p.clone().expect("p node should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("p", Vec::new())),
            p.clone().expect("p node should exist").borrow().node_kind()
        );

        assert!(p
            .clone()
            .expect("p node should exist")
            .borrow()
            .first_child()
            .is_none());
        assert!(p
            .expect("p node should exist")
            .borrow()
            .next_sibling()
            .is_none());
    }
}
