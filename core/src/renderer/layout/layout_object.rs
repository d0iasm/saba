//! https://www.w3.org/TR/css-box-3/
//! https://www.w3.org/TR/css-layout-api-1/
//! https://www.w3.org/TR/css3-linebox/
//! https://www.w3.org/TR/css-position-3/

use crate::alloc::string::ToString;
use crate::browser::Browser;
use crate::constants::CHAR_HEIGHT_WITH_PADDING;
use crate::constants::CHAR_WIDTH;
use crate::constants::CONTENT_AREA_WIDTH;
use crate::display_item::DisplayItem;
use crate::renderer::css::cssom::ComponentValue;
use crate::renderer::css::cssom::Declaration;
use crate::renderer::css::cssom::Selector;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::css::token::CssToken;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::layout::color::Color;
use crate::renderer::layout::computed_style::BoxInfo;
use crate::renderer::layout::computed_style::ComputedStyle;
use crate::renderer::layout::computed_style::DisplayType;
use crate::renderer::layout::computed_style::FontSize;
use crate::renderer::layout::layout_point::LayoutPoint;
use crate::renderer::layout::layout_size::LayoutSize;
use crate::utils::console_error;
use alloc::format;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn create_layout_object(
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

            let display_type = layout_object.borrow().style().display();
            if display_type == DisplayType::DisplayNone {
                return None;
            }

            let kind = layout_object.borrow().kind();
            layout_object
                .borrow_mut()
                .set_kind(layout_object_kind_from_display(kind, display_type));
            Some(layout_object)
        }
        None => None,
    }
}

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
            // Handle a <body> as a block element for simplicity.
            if e.is_block_element() || e.kind() == ElementKind::Body {
                LayoutObjectKind::Block
            } else {
                LayoutObjectKind::Inline
            }
        }
        NodeKind::Text(_) => LayoutObjectKind::Text,
    }
}

fn layout_object_kind_from_display(
    kind: LayoutObjectKind,
    display: DisplayType,
) -> LayoutObjectKind {
    if kind == LayoutObjectKind::Text {
        return kind;
    }

    match display {
        DisplayType::Block => LayoutObjectKind::Block,
        DisplayType::Inline => LayoutObjectKind::Inline,
        DisplayType::DisplayNone => panic!("should not be reached here"),
    }
}

#[derive(Debug, Clone)]
pub struct LayoutObject {
    browser: Weak<RefCell<Browser>>,
    kind: LayoutObjectKind,
    // Similar structure with a DOM node.
    node: Rc<RefCell<Node>>,
    pub first_child: Option<Rc<RefCell<LayoutObject>>>,
    pub next_sibling: Option<Rc<RefCell<LayoutObject>>>,
    // CSS information.
    style: ComputedStyle,
    // Layout information.
    // https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_box.h;drc=48340c1e35efad5fb0253025dcc36b3a9573e258;bpv=1;bpt=1;l=2401
    point: LayoutPoint,
    // https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_box.h;drc=48340c1e35efad5fb0253025dcc36b3a9573e258;bpv=1;bpt=1;l=2404
    size: LayoutSize,
}

impl LayoutObject {
    fn new(browser: Weak<RefCell<Browser>>, node: Rc<RefCell<Node>>) -> Self {
        Self {
            browser,
            kind: layout_object_kind_by_node(&node),
            node: node.clone(),
            first_child: None,
            next_sibling: None,
            style: ComputedStyle::new(),
            point: LayoutPoint::new(0, 0),
            size: LayoutSize::new(0, 0),
        }
    }

    pub fn node(&self) -> Rc<RefCell<Node>> {
        self.node.clone()
    }

    pub fn set_kind(&mut self, kind: LayoutObjectKind) {
        self.kind = kind;
    }

    pub fn kind(&self) -> LayoutObjectKind {
        self.kind
    }

    pub fn node_kind(&self) -> NodeKind {
        self.node.borrow().kind().clone()
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.first_child.as_ref().cloned()
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.next_sibling.as_ref().cloned()
    }

    pub fn style(&self) -> ComputedStyle {
        self.style.clone()
    }

    pub fn point(&self) -> LayoutPoint {
        self.point
    }

    pub fn size(&self) -> LayoutSize {
        self.size
    }

    /// https://www.w3.org/TR/css-cascade-4/#cascading
    /// Cascading yields the cascaded value. It takes takes an unordered list of declared values
    /// and outputs a single cascaded value for a property.
    // It doens't implement https://www.w3.org/TR/css-cascade-4/#cascade-sort properly
    // because it supports "Normal user declarations" input only.
    pub fn cascading_style(&mut self, declarations: Vec<Declaration>) {
        for declaration in declarations {
            match declaration.property.as_str() {
                "background-color" => {
                    if let ComponentValue::Keyword(value) = &declaration.value {
                        let color = match Color::from_name(value) {
                            Ok(color) => color,
                            Err(e) => {
                                console_error(&self.browser, format!("{:?}", e));
                                Color::white()
                            }
                        };
                        self.style.set_background_color(color);
                        continue;
                    }

                    if let ComponentValue::PreservedToken(CssToken::HashToken(color_code)) =
                        &declaration.value
                    {
                        let color = match Color::from_code(color_code) {
                            Ok(color) => color,
                            Err(e) => {
                                console_error(&self.browser, format!("{:?}", e));
                                Color::white()
                            }
                        };
                        self.style.set_background_color(color);
                        continue;
                    }
                }
                "color" => {
                    if let ComponentValue::Keyword(value) = &declaration.value {
                        let color = match Color::from_name(value) {
                            Ok(color) => color,
                            Err(e) => {
                                console_error(&self.browser, format!("{:?}", e));
                                Color::black()
                            }
                        };
                        self.style.set_color(color);
                    }

                    if let ComponentValue::PreservedToken(CssToken::HashToken(color_code)) =
                        &declaration.value
                    {
                        let color = match Color::from_code(color_code) {
                            Ok(color) => color,
                            Err(e) => {
                                console_error(&self.browser, format!("{:?}", e));
                                Color::black()
                            }
                        };
                        self.style.set_color(color);
                    }
                }
                "display" => {
                    if let ComponentValue::Keyword(value) = declaration.value {
                        let display_type = match DisplayType::from_str(&value) {
                            Ok(display_type) => display_type,
                            Err(e) => {
                                console_error(&self.browser, format!("{:?}", e));
                                DisplayType::DisplayNone
                            }
                        };
                        self.style.set_display(display_type)
                    }
                }
                "height" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        // TODO: remove this? because layout() updates size and style.
                        self.size.set_height(value as i64);
                        self.style.set_height(value);
                    }
                }
                "width" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        // TODO: remove this? because layout() updates size and style.
                        self.size.set_width(value as i64);
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
                _ => {
                    /*
                    console_warning(
                    &self.browser,
                    format!("css property {} is not supported yet", declaration.property),
                    );
                    */
                }
            }
        }
    }

    /// https://www.w3.org/TR/css-cascade-4/#defaulting
    pub fn defaulting_style(&mut self, node: &Rc<RefCell<Node>>) {
        self.style.defaulting(node);
    }

    /// https://www.w3.org/TR/css-cascade-4/#inheriting
    /// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/css/resolver/style_resolver.h;drc=48340c1e35efad5fb0253025dcc36b3a9573e258;bpv=1;bpt=1;l=234
    pub fn inherit_style(&mut self, parent_style: &ComputedStyle) {
        // This may be a hacky way to inherit.
        if self.kind() == LayoutObjectKind::Text {
            // Now, only text object inherits CSS properties from its parent.
            self.style.inherit(parent_style);
        }
    }

    /// Returns the size of this element including margins, paddings, etc.
    pub fn compute_size(&mut self, parent_size: LayoutSize) {
        let mut size = LayoutSize::new(0, 0);
        let mut is_height_set = false;
        let mut is_width_set = false;

        if self.style.height() != 0.0 {
            is_height_set = true;
            size.set_height(self.style.height() as i64);
        }
        if self.style.width() != 0.0 {
            is_width_set = true;
            size.set_width(self.style.width() as i64);
        }

        if is_height_set && is_width_set {
            return;
        }

        match self.kind() {
            LayoutObjectKind::Block => {
                // For a block element, consider the parent's width.
                // TODO: add content_size to LayoutSize?
                size.set_width(
                    parent_size.width()
                        - self.style.padding_left() as i64
                        - self.style.padding_right() as i64,
                );

                // For height, sum up the height of all children directly under this element.
                let mut height = 0;
                let mut child = self.first_child();
                while child.is_some() {
                    let c = match child {
                        Some(c) => c,
                        None => panic!("first child should exist"),
                    };

                    height += c.borrow().size.height();

                    child = c.borrow().next_sibling();
                }
                size.set_height(height);
            }
            LayoutObjectKind::Inline => {
                // Sum up the width and height of all children directly under this element.
                let mut width = 0;
                let mut height = 0;
                let mut child = self.first_child();
                while child.is_some() {
                    let c = match child {
                        Some(c) => c,
                        None => panic!("first child should exist"),
                    };

                    width += c.borrow().size.width();
                    height += c.borrow().size.height();

                    child = c.borrow().next_sibling();
                }

                size.set_width(width);
                size.set_height(height);
            }
            LayoutObjectKind::Text => {
                if let NodeKind::Text(t) = self.node_kind() {
                    let ratio = match self.style.font_size() {
                        FontSize::Medium => 1,
                        FontSize::XLarge => 2,
                        FontSize::XXLarge => 3,
                    };
                    let width = CHAR_WIDTH * ratio * t.len() as i64;
                    if width > CONTENT_AREA_WIDTH {
                        // The text is multiple lines.
                        size.set_width(CONTENT_AREA_WIDTH);
                        let line_num = if width.wrapping_rem(CONTENT_AREA_WIDTH) == 0 {
                            width.wrapping_div(CONTENT_AREA_WIDTH)
                        } else {
                            width.wrapping_div(CONTENT_AREA_WIDTH) + 1
                        };
                        size.set_height(CHAR_HEIGHT_WITH_PADDING * ratio * line_num);
                    } else {
                        // The text is signle line.
                        size.set_width(width);
                        size.set_height(CHAR_HEIGHT_WITH_PADDING * ratio);
                    }
                }
            }
        }

        self.size = size;
    }

    /// Returns the position of this element.
    ///
    /// The position is calculated based on the normal flow, which is the default value in the `position` property in CSS.
    /// https://developer.mozilla.org/en-US/docs/Learn/CSS/CSS_layout/Normal_Flow
    pub fn compute_position(
        &mut self,
        parent_point: LayoutPoint,
        previous_sibiling_point: Option<LayoutPoint>,
        previous_sibiling_size: Option<LayoutSize>,
    ) {
        let mut point = LayoutPoint::new(0, 0);

        match self.kind() {
            LayoutObjectKind::Block => {
                if let (Some(size), Some(pos)) = (previous_sibiling_size, previous_sibiling_point) {
                    // TODO: consider padding of the previous sibiling.
                    point.set_y(
                        point.y() + pos.y() + size.height() + self.style.margin_top() as i64,
                    );
                } else {
                    point.set_y(parent_point.y());
                }
            }
            // TODO: calculate the position for inline elements.
            LayoutObjectKind::Inline => {}
            LayoutObjectKind::Text => {}
        }

        self.point = point;
    }

    pub fn is_node_selected(&self, selector: &Selector) -> bool {
        match &self.node_kind() {
            NodeKind::Element(e) => match selector {
                Selector::TypeSelector(type_name) => {
                    if e.kind().to_string() == *type_name {
                        return true;
                    }
                    false
                }
                Selector::ClassSelector(class_name) => {
                    for attr in &e.attributes() {
                        if attr.name() == "class" && attr.value() == *class_name {
                            return true;
                        }
                    }
                    false
                }
                Selector::IdSelector(id_name) => {
                    for attr in &e.attributes() {
                        if attr.name() == "id" && attr.value() == *id_name {
                            return true;
                        }
                    }
                    false
                }
                Selector::UnknownSelector => false,
            },
            _ => false,
        }
    }

    /// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_object.h;drc=0e9a0b6e9bb6ec59521977eec805f5d0bca833e0;bpv=1;bpt=1;l=2377
    pub fn paint(&mut self) -> Option<DisplayItem> {
        if self.style.display() == DisplayType::DisplayNone {
            return None;
        }

        match self.kind {
            LayoutObjectKind::Block => {
                if let NodeKind::Element(_e) = self.node_kind() {
                    return Some(DisplayItem::Rect {
                        style: self.style(),
                        layout_point: self.point(),
                        layout_size: self.size(),
                    });
                }
            }
            LayoutObjectKind::Inline => {
                if let NodeKind::Element(e) = self.node_kind() {
                    if e.kind() == ElementKind::A {
                        // <a> element should have a text node as a first child
                        let text_node = self.first_child();
                        let mut link_text = String::new();
                        if let Some(text_node) = text_node {
                            match text_node.borrow().node_kind() {
                                NodeKind::Text(text) => link_text = text,
                                _ => return None,
                            }
                        }

                        let mut href = String::new();
                        for attr in e.attributes() {
                            if attr.name() == "href" {
                                href = attr.value()
                            }
                        }

                        // remove the first child from the tree to avoid operating it twice
                        self.first_child = None;
                        return Some(DisplayItem::Link {
                            text: link_text,
                            destination: href,
                            style: self.style(),
                            layout_point: self.point(),
                        });
                    }
                    if e.kind() == ElementKind::IMG {
                        for attr in &e.attributes() {
                            if attr.name() == "src" {
                                return Some(DisplayItem::Img {
                                    src: attr.value(),
                                    style: self.style(),
                                    layout_point: self.point(),
                                });
                            }
                        }
                    }
                }
            }
            LayoutObjectKind::Text => {
                if let NodeKind::Text(t) = self.node_kind() {
                    return Some(DisplayItem::Text {
                        text: t,
                        style: self.style(),
                        layout_point: self.point(),
                    });
                }
            }
        }

        None
    }
}
