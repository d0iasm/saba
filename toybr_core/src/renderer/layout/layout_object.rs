//! https://www.w3.org/TR/css-box-3/
//! https://www.w3.org/TR/css-layout-api-1/

use crate::renderer::css::cssom::*;
use crate::renderer::css::token::CssToken;
use crate::renderer::html::dom::*;
use crate::renderer::layout::color::*;
use crate::renderer::layout::computed_style::*;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct LayoutObject {
    // Similar structure with Node in renderer/dom.rs.
    node: Rc<RefCell<Node>>,
    pub first_child: Option<Rc<RefCell<LayoutObject>>>,
    pub next_sibling: Option<Rc<RefCell<LayoutObject>>>,
    // CSS information.
    pub style: ComputedStyle,
    // Layout information.
    pub position: LayoutPosition,
}

impl LayoutObject {
    pub fn new(node: Rc<RefCell<Node>>) -> Self {
        Self {
            node: node.clone(),
            first_child: None,
            next_sibling: None,
            style: ComputedStyle::new(&node),
            position: LayoutPosition::new(0.0, 0.0),
        }
    }

    pub fn paint(&self) {}

    pub fn node(&self) -> Rc<RefCell<Node>> {
        self.node.clone()
    }

    pub fn kind(&self) -> NodeKind {
        self.node.borrow().kind().clone()
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.first_child.as_ref().map(|n| n.clone())
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.next_sibling.as_ref().map(|n| n.clone())
    }

    pub fn set_style(&mut self, declarations: Vec<Declaration>) {
        for declaration in declarations {
            match declaration.property.as_str() {
                "background-color" => {
                    if let ComponentValue::Keyword(value) = &declaration.value {
                        self.style.set_background_color(Color::from_name(value));
                    }

                    if let ComponentValue::InputToken(value) = &declaration.value {
                        if let CssToken::HashToken(color_code) = value {
                            self.style
                                .set_background_color(Color::from_code(color_code));
                        }
                    }
                }
                "color" => {
                    if let ComponentValue::Keyword(value) = &declaration.value {
                        self.style.set_color(Color::from_name(value));
                    }

                    if let ComponentValue::InputToken(value) = &declaration.value {
                        if let CssToken::HashToken(color_code) = value {
                            self.style.set_color(Color::from_code(color_code));
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
                _ => println!(
                    "warning: css property {} is not supported yet",
                    declaration.property,
                ),
            }
        }
    }

    pub fn layout(&mut self, parent_style: &ComputedStyle, parent_position: &LayoutPosition) {
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
        match &self.kind() {
            NodeKind::Element(e) => match selector {
                Selector::TypeSelector(type_name) => {
                    if e.kind().to_string() == *type_name {
                        return true;
                    }
                    return false;
                }
                Selector::ClassSelector(class_name) => {
                    for attr in &e.attributes() {
                        if attr.name == "class" && attr.value == *class_name {
                            return true;
                        }
                    }
                    return false;
                }
                Selector::IdSelector(id_name) => {
                    for attr in &e.attributes() {
                        if attr.name == "id" && attr.value == *id_name {
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
}
