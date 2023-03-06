//use crate::css::cssom::*;
//use crate::html::dom::*;
use crate::layout::layout_object::LayoutObject;
use alloc::rc::Rc;
use core::cell::RefCell;

struct FrameView {
    root: Option<Rc<RefCell<LayoutObject>>>,
}

impl FrameView {
    /*
    pub fn new(root: Rc<RefCell<Node>>, cssom: &StyleSheet) -> Self {
        Self {
            root: Self::create_render_tree(&Some(root), &None, cssom),
        }
    }
    */
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn update_layout(&self) {}

    pub fn paint(&self) {}
}
