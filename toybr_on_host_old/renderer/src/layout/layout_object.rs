//! https://www.w3.org/TR/css-box-3/
//! https://www.w3.org/TR/css-layout-api-1/

use crate::css::cssom::*;
use crate::css::token::CssToken;
use crate::html::dom::*;
use crate::layout::color::*;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

/// https://w3c.github.io/csswg-drafts/css-text/#white-space-property
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WhiteSpace {
    Normal,
    Pre,
}

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    background_color: Option<Color>,
    color: Option<Color>,
    display: DisplayType,
    height: Option<f64>,
    width: Option<f64>,
    margin: Option<BoxInfo>,
    padding: Option<BoxInfo>,
    font_size: Option<FontSize>,
    white_space: WhiteSpace,
}

impl ComputedStyle {
    pub fn new(node: &Rc<RefCell<Node>>) -> Self {
        Self {
            background_color: None,
            color: None,
            display: Self::default_display_type(node),
            width: None,
            height: None,
            margin: None,
            padding: None,
            font_size: Self::default_font_size(node),
            white_space: Self::default_white_space(node),
        }
    }

    fn default_display_type(node: &Rc<RefCell<Node>>) -> DisplayType {
        match &node.borrow().kind() {
            NodeKind::Document => DisplayType::Block,
            NodeKind::Element(element) => match element.kind() {
                ElementKind::Html
                | ElementKind::Body
                | ElementKind::Div
                | ElementKind::Ul
                | ElementKind::Li
                | ElementKind::H1
                | ElementKind::P => DisplayType::Block,
                ElementKind::Script | ElementKind::Head | ElementKind::Style => {
                    DisplayType::DisplayNone
                }
                _ => DisplayType::Inline,
            },
            NodeKind::Text(_) => DisplayType::Inline,
        }
    }

    fn default_font_size(node: &Rc<RefCell<Node>>) -> Option<FontSize> {
        match &node.borrow().kind() {
            NodeKind::Element(element) => match element.kind() {
                ElementKind::H1 => Some(FontSize::XXLarge),
                ElementKind::H2 => Some(FontSize::XLarge),
                _ => None,
            },
            _ => None,
        }
    }

    fn default_white_space(node: &Rc<RefCell<Node>>) -> WhiteSpace {
        match &node.borrow().kind() {
            NodeKind::Element(element) => match element.kind() {
                ElementKind::P => WhiteSpace::Normal,
                ElementKind::Pre => WhiteSpace::Pre,
                _ => WhiteSpace::Normal,
            },
            _ => WhiteSpace::Normal,
        }
    }

    pub fn inherit(&mut self, parent_style: &ComputedStyle) {
        if self.color.is_none() {
            self.color = Some(parent_style.color().clone());
        }
        if self.background_color.is_none() {
            self.background_color = Some(parent_style.background_color().clone());
        }
        if self.height.is_none() {
            self.height = Some(parent_style.height().clone());
        }
        if self.width.is_none() {
            self.width = Some(parent_style.width().clone());
        }
        if self.margin.is_none() {
            self.margin = Some(parent_style.margin().clone());
        }
        if self.padding.is_none() {
            self.padding = Some(parent_style.padding().clone());
        }
        if self.font_size.is_none() {
            self.font_size = Some(parent_style.font_size().clone());
        }

        // TODO: check if it's ok to inherit parent white space always
        self.white_space = parent_style.white_space();
    }

    pub fn background_color(&self) -> Color {
        if let Some(ref bc) = self.background_color {
            bc.clone()
        } else {
            Color::from_name("white")
        }
    }

    pub fn color(&self) -> Color {
        if let Some(ref c) = self.color {
            c.clone()
        } else {
            Color::from_name("black")
        }
    }

    pub fn height(&self) -> f64 {
        if let Some(h) = self.height {
            h
        } else {
            0f64
        }
    }

    pub fn display(&self) -> DisplayType {
        self.display
    }

    pub fn width(&self) -> f64 {
        if let Some(w) = self.width {
            w
        } else {
            // 1200 is a default value defined at src/gui/browser_window/window.ui
            1200.0f64
        }
    }

    pub fn margin(&self) -> BoxInfo {
        if let Some(ref m) = self.margin {
            m.clone()
        } else {
            BoxInfo::new(0.0, 0.0, 0.0, 0.0)
        }
    }

    pub fn padding(&self) -> BoxInfo {
        if let Some(ref p) = self.padding {
            p.clone()
        } else {
            BoxInfo::new(0.0, 0.0, 0.0, 0.0)
        }
    }

    pub fn font_size(&self) -> FontSize {
        if let Some(ref s) = self.font_size {
            s.clone()
        } else {
            FontSize::Medium
        }
    }

    pub fn white_space(&self) -> WhiteSpace {
        self.white_space
    }

    pub fn margin_top(&self) -> f64 {
        self.margin().top
    }

    pub fn margin_left(&self) -> f64 {
        self.margin().left
    }

    pub fn margin_right(&self) -> f64 {
        self.margin().right
    }

    pub fn margin_bottom(&self) -> f64 {
        self.margin().bottom
    }

    pub fn padding_top(&self) -> f64 {
        self.padding().top
    }

    pub fn padding_left(&self) -> f64 {
        self.padding().left
    }

    pub fn padding_right(&self) -> f64 {
        self.padding().right
    }

    pub fn padding_bottom(&self) -> f64 {
        self.padding().bottom
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DisplayType {
    /// https://www.w3.org/TR/css-display-3/#valdef-display-block
    Block,
    /// https://www.w3.org/TR/css-display-3/#valdef-display-inline
    Inline,
    /// https://www.w3.org/TR/css-display-3/#valdef-display-none
    DisplayNone,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxInfo {
    top: f64,
    right: f64,
    left: f64,
    bottom: f64,
}

impl BoxInfo {
    fn new(top: f64, right: f64, left: f64, bottom: f64) -> Self {
        Self {
            top,
            right,
            left,
            bottom,
        }
    }
}

/// https://www.w3.org/TR/css-fonts-4/#absolute-size-mapping
/// https://docs.gtk.org/Pango/pango_markup.html
/// align with pango markup syntax
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FontSize {
    Medium,
    XLarge,
    XXLarge,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutPosition {
    x: f64,
    y: f64,
}

impl LayoutPosition {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

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
                        self.style.background_color = Some(Color::from_name(value));
                    }

                    if let ComponentValue::InputToken(value) = &declaration.value {
                        if let CssToken::HashToken(color_code) = value {
                            self.style.background_color = Some(Color::from_code(color_code));
                        }
                    }
                }
                "color" => {
                    if let ComponentValue::Keyword(value) = &declaration.value {
                        self.style.color = Some(Color::from_name(value));
                    }

                    if let ComponentValue::InputToken(value) = &declaration.value {
                        if let CssToken::HashToken(color_code) = value {
                            self.style.color = Some(Color::from_code(color_code));
                        }
                    }
                }
                "height" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.height = Some(value);
                    }
                }
                "width" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.width = Some(value);
                    }
                }
                "margin" => {
                    // TODO: support string (e.g. "auto")
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.margin = Some(BoxInfo::new(value, value, value, value));
                    }
                }
                "margin-top" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.margin = match &self.style.margin {
                            Some(m) => Some(BoxInfo::new(value, m.right, m.bottom, m.left)),
                            None => Some(BoxInfo::new(value, 0.0, 0.0, 0.0)),
                        };
                    }
                }
                "margin-right" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.margin = match &self.style.margin {
                            Some(m) => Some(BoxInfo::new(m.top, value, m.bottom, m.left)),
                            None => Some(BoxInfo::new(0.0, value, 0.0, 0.0)),
                        };
                    }
                }
                "margin-bottom" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.margin = match &self.style.margin {
                            Some(m) => Some(BoxInfo::new(m.top, m.right, value, m.left)),
                            None => Some(BoxInfo::new(0.0, 0.0, value, 0.0)),
                        };
                    }
                }
                "margin-left" => {
                    if let ComponentValue::Number(value) = declaration.value {
                        self.style.margin = match &self.style.margin {
                            Some(m) => Some(BoxInfo::new(m.top, m.right, m.bottom, value)),
                            None => Some(BoxInfo::new(0.0, 0.0, 0.0, value)),
                        };
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
        match parent_style.display {
            DisplayType::Inline => {
                match self.style.display() {
                    DisplayType::Block => {
                        // TODO: set position property
                        self.position.x = self.style.margin().left;
                        self.position.y = self.style.margin().top + parent_style.height();
                    }
                    DisplayType::Inline => {
                        self.position.x = parent_position.x + parent_style.width();
                        self.position.y = parent_position.y;
                    }
                    DisplayType::DisplayNone => {}
                }
            }
            DisplayType::Block => {
                match self.style.display() {
                    DisplayType::Block => {
                        self.position.x = self.style.margin().left;
                        self.position.y = parent_position.y
                            + parent_style.height()
                            + parent_style.margin().bottom
                            + self.style.margin().top;
                    }
                    DisplayType::Inline => {
                        // TODO: set position property
                        self.position.x = 0.0;
                        self.position.y = parent_style.height();
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
