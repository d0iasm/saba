//! https://www.w3.org/TR/css-box-3/
//! https://www.w3.org/TR/css-layout-api-1/
//! https://www.w3.org/TR/css3-linebox/
//! https://www.w3.org/TR/css-position-3/

use crate::browser::Browser;
use crate::common::ui::UiObject;
use crate::renderer::css::cssom::*;
use crate::renderer::css::token::CssToken;
use crate::renderer::html::dom::*;
use crate::renderer::layout::color::*;
use crate::renderer::layout::computed_style::*;
use crate::utils::*;
use alloc::rc::{Rc, Weak};
use alloc::vec::Vec;
use core::cell::RefCell;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayoutObjectKind {
    Block,
    Inline,
    Text,
}

fn layout_object_kind_by_node(node: &Rc<RefCell<Node>>) -> LayoutObjectKind {
    match node.borrow().kind() {
        NodeKind::Document => panic!("should not create a layout object for a Document node"),
        NodeKind::Element(e) => {
            if e.is_block_element() {
                LayoutObjectKind::Block
            } else {
                LayoutObjectKind::Inline
            }
        }
        NodeKind::Text(_) => LayoutObjectKind::Text,
    }
}

#[derive(Debug, Clone)]
pub struct LayoutObject<U: UiObject> {
    browser: Weak<RefCell<Browser<U>>>,
    kind: LayoutObjectKind,
    // Similar structure with Node in renderer/dom.rs.
    node: Rc<RefCell<Node>>,
    pub first_child: Option<Rc<RefCell<LayoutObject<U>>>>,
    pub next_sibling: Option<Rc<RefCell<LayoutObject<U>>>>,
    // CSS information.
    pub style: ComputedStyle,
    // Layout information.
    pub position: LayoutPosition,
}

impl<U: UiObject> LayoutObject<U> {
    pub fn new(browser: Weak<RefCell<Browser<U>>>, node: Rc<RefCell<Node>>) -> Self {
        Self {
            browser,
            kind: layout_object_kind_by_node(&node),
            node: node.clone(),
            first_child: None,
            next_sibling: None,
            style: ComputedStyle::new(&node),
            position: LayoutPosition::new(0.0, 0.0),
        }
    }

    pub fn browser(&self) -> Weak<RefCell<Browser<U>>> {
        self.browser.clone()
    }

    pub fn node(&self) -> Rc<RefCell<Node>> {
        self.node.clone()
    }

    pub fn kind(&self) -> LayoutObjectKind {
        self.kind.clone()
    }

    pub fn node_kind(&self) -> NodeKind {
        self.node.borrow().kind().clone()
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<LayoutObject<U>>>> {
        self.first_child.as_ref().map(|n| n.clone())
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<LayoutObject<U>>>> {
        self.next_sibling.as_ref().map(|n| n.clone())
    }

    pub fn style(&self) -> ComputedStyle {
        self.style.clone()
    }

    pub fn position(&self) -> LayoutPosition {
        self.position.clone()
    }

    pub fn set_style(&mut self, declarations: Vec<Declaration>) {
        for declaration in declarations {
            match declaration.property.as_str() {
                "background-color" => {
                    if let ComponentValue::Keyword(value) = &declaration.value {
                        let color = match Color::from_name(value) {
                            Ok(color) => color,
                            Err(e) => {
                                console_error(self.browser.clone(), format!("{:?}", e));
                                Color::white()
                            }
                        };
                        self.style.set_background_color(color);
                    }

                    if let ComponentValue::InputToken(value) = &declaration.value {
                        if let CssToken::HashToken(color_code) = value {
                            let color = match Color::from_code(color_code) {
                                Ok(color) => color,
                                Err(e) => {
                                    console_error(self.browser.clone(), format!("{:?}", e));
                                    Color::white()
                                }
                            };
                            self.style.set_background_color(color);
                        }
                    }
                }
                "color" => {
                    if let ComponentValue::Keyword(value) = &declaration.value {
                        let color = match Color::from_name(value) {
                            Ok(color) => color,
                            Err(e) => {
                                console_error(self.browser.clone(), format!("{:?}", e));
                                Color::black()
                            }
                        };
                        self.style.set_color(color);
                    }

                    if let ComponentValue::InputToken(value) = &declaration.value {
                        if let CssToken::HashToken(color_code) = value {
                            let color = match Color::from_code(color_code) {
                                Ok(color) => color,
                                Err(e) => {
                                    console_error(self.browser.clone(), format!("{:?}", e));
                                    Color::black()
                                }
                            };
                            self.style.set_color(color);
                        }
                    }
                }
                "height" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.set_height(value);
                    }
                }
                "width" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.set_width(value);
                    }
                }
                "margin" => {
                    // TODO: support string (e.g. "auto")
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style
                            .set_margin(BoxInfo::new(value, value, value, value));
                    }
                }
                "margin-top" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        let m = self.style.margin();
                        self.style
                            .set_margin(BoxInfo::new(value, m.right(), m.bottom(), m.left()));
                    }
                }
                "margin-right" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        let m = self.style.margin();
                        self.style
                            .set_margin(BoxInfo::new(m.top(), value, m.bottom(), m.left()));
                    }
                }
                "margin-bottom" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        let m = self.style.margin();
                        self.style
                            .set_margin(BoxInfo::new(m.top(), m.right(), value, m.left()));
                    }
                }
                "margin-left" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        let m = self.style.margin();
                        self.style
                            .set_margin(BoxInfo::new(m.top(), m.right(), m.bottom(), value));
                    }
                }
                // TODO: support padding
                _ => console_warning(
                    self.browser.clone(),
                    format!("css property {} is not supported yet", declaration.property,),
                ),
            }
        }
    }

    /// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_object.h;drc=0e9a0b6e9bb6ec59521977eec805f5d0bca833e0;bpv=1;bpt=1;l=2398
    pub fn update_layout(
        &mut self,
        parent_style: &ComputedStyle,
        parent_position: &LayoutPosition,
    ) {
        match parent_style.display() {
            DisplayType::Inline => {
                match self.style.display() {
                    DisplayType::Block => {
                        // TODO: set position property
                        self.position.set_x(self.style.margin().left());
                        self.position
                            .set_y(self.style.margin().top() + parent_style.height());
                    }
                    DisplayType::Inline => {
                        self.position
                            .set_x(parent_position.x() + parent_style.width());
                        self.position.set_y(parent_position.y());
                    }
                    DisplayType::DisplayNone => {}
                }
            }
            DisplayType::Block => {
                match self.style.display() {
                    DisplayType::Block => {
                        self.position.set_x(self.style.margin().left());
                        self.position.set_y(
                            parent_position.y()
                                + parent_style.height()
                                + parent_style.margin().bottom()
                                + self.style.margin().top(),
                        );
                    }
                    DisplayType::Inline => {
                        // TODO: set position property
                        self.position.set_x(0.0);
                        self.position.set_y(parent_style.height());
                    }
                    DisplayType::DisplayNone => {}
                }
            }
            DisplayType::DisplayNone => {}
        }
    }

    pub fn is_node_selected(&self, selector: &Selector) -> bool {
        match &self.node_kind() {
            NodeKind::Element(e) => match selector {
                Selector::TypeSelector(type_name) => {
                    if e.kind().to_string() == *type_name {
                        return true;
                    }
                    return false;
                }
                Selector::ClassSelector(class_name) => {
                    for attr in &e.attributes() {
                        if attr.name() == "class" && attr.value() == *class_name {
                            return true;
                        }
                    }
                    return false;
                }
                Selector::IdSelector(id_name) => {
                    for attr in &e.attributes() {
                        if attr.name() == "id" && attr.value() == *id_name {
                            return true;
                        }
                    }
                    return false;
                }
                Selector::UnknownSelector => false,
            },
            _ => false,
        }
    }

    /// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_object.h;drc=0e9a0b6e9bb6ec59521977eec805f5d0bca833e0;bpv=1;bpt=1;l=2377
    pub fn paint(&mut self) {
        match self.node_kind() {
            NodeKind::Document => {}
            NodeKind::Element(e) => match e.kind() {
                ElementKind::A => {
                    let text_node = self.first_child();
                    let mut link_text = String::new();
                    if let Some(text_node) = text_node {
                        match text_node.borrow().node_kind() {
                            NodeKind::Text(text) => link_text = text,
                            _ => return,
                        }
                    }

                    let mut href = String::new();
                    for attr in e.attributes() {
                        if attr.name() == "href" {
                            href = attr.value()
                        }
                    }

                    add_link_display_item(self, href, link_text);

                    self.first_child = None;
                }
                _ => {}
            },
            NodeKind::Text(text) => {
                add_text_display_item(self, text);
            }
        }
    }
}
