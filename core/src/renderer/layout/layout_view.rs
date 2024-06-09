//! https://www.w3.org/TR/css-box-3/
//! https://www.w3.org/TR/css-layout-api-1/
//! https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_view.h

use crate::browser::Browser;
use crate::constants::CONTENT_AREA_WIDTH;
use crate::display_item::DisplayItem;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::api::get_target_element_node;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::layout::computed_style::*;
use crate::renderer::layout::layout_object::LayoutObject;
use crate::renderer::layout::layout_object::LayoutObjectKind;
use crate::renderer::layout::layout_point::LayoutPoint;
use crate::renderer::layout::layout_size::LayoutSize;
use alloc::rc::{Rc, Weak};
use alloc::vec::Vec;
use core::cell::RefCell;

fn create_layout_object(
    browser: Weak<RefCell<Browser>>,
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    match node {
        Some(n) => {
            let layout_object =
                Rc::new(RefCell::new(LayoutObject::new(browser.clone(), n.clone())));

            // Apply CSS rules to LayoutObject.
            for rule in &cssom.rules {
                if layout_object.borrow().is_node_selected(&rule.selector) {
                    layout_object
                        .borrow_mut()
                        .cascading_style(rule.declarations.clone());
                }
            }

            // Apply a default value to a property.
            {
                layout_object.borrow_mut().defaulting_style(n);
            }

            // Inherit a parent CSS style.
            if let Some(parent) = parent_obj {
                layout_object
                    .borrow_mut()
                    .inherit_style(&parent.borrow().style());
            }

            if layout_object.borrow().style().display() == DisplayType::DisplayNone {
                return None;
            }

            Some(layout_object)
        }
        None => None,
    }
}

/// Converts DOM tree to render tree.
fn build_layout_tree(
    browser: Weak<RefCell<Browser>>,
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    let layout_object = create_layout_object(browser.clone(), node, parent_obj, cssom);

    layout_object.as_ref()?;

    if let Some(n) = node {
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
        obj.borrow_mut().first_child = first_child;
        obj.borrow_mut().next_sibling = next_sibling;
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
        previous_sibiling_point: Option<LayoutPoint>,
        previous_sibiling_size: Option<LayoutSize>,
    ) {
        match node {
            Some(n) => {
                n.borrow_mut().compute_position(
                    parent_point,
                    previous_sibiling_point,
                    previous_sibiling_size,
                );

                let first_child = n.borrow().first_child();
                Self::calculate_node_position(&first_child, n.borrow().point(), None, None);

                let next_sibling = n.borrow().next_sibling();
                Self::calculate_node_position(
                    &next_sibling,
                    parent_point,
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

        Self::calculate_node_position(&self.root, LayoutPoint::new(0, 0), None, None);
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
}
