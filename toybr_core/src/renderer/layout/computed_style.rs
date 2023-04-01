//! https://developer.mozilla.org/en-US/docs/Web/CSS/computed_value

use crate::renderer::html::dom::{ElementKind, Node, NodeKind};
use crate::renderer::layout::color::*;
use alloc::rc::Rc;
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
            display: default_display_type(node),
            width: None,
            height: None,
            margin: None,
            padding: None,
            font_size: default_font_size(node),
            white_space: default_white_space(node),
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

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = Some(color);
    }

    pub fn background_color(&self) -> Color {
        if let Some(ref bc) = self.background_color {
            bc.clone()
        } else {
            Color::from_name("white").unwrap()
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = Some(color);
    }

    pub fn color(&self) -> Color {
        if let Some(ref c) = self.color {
            c.clone()
        } else {
            Color::from_name("black").unwrap()
        }
    }

    pub fn set_height(&mut self, height: f64) {
        self.height = Some(height);
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

    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    pub fn width(&self) -> f64 {
        if let Some(w) = self.width {
            w
        } else {
            // 1200 is a default value defined at src/gui/browser_window/window.ui
            1200.0f64
        }
    }

    pub fn set_margin(&mut self, margin: BoxInfo) {
        self.margin = Some(margin);
    }

    pub fn margin(&self) -> BoxInfo {
        if let Some(ref m) = self.margin {
            m.clone()
        } else {
            BoxInfo::new(0.0, 0.0, 0.0, 0.0)
        }
    }

    pub fn set_padding(&mut self, padding: BoxInfo) {
        self.padding = Some(padding);
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
    pub fn new(top: f64, right: f64, left: f64, bottom: f64) -> Self {
        Self {
            top,
            right,
            left,
            bottom,
        }
    }

    pub fn top(&self) -> f64 {
        self.top
    }

    pub fn right(&self) -> f64 {
        self.right
    }

    pub fn left(&self) -> f64 {
        self.left
    }

    pub fn bottom(&self) -> f64 {
        self.bottom
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

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn set_x(&mut self, x: f64) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: f64) {
        self.y = y;
    }
}

fn default_display_type(node: &Rc<RefCell<Node>>) -> DisplayType {
    match &node.borrow().kind() {
        NodeKind::Document => DisplayType::Inline,
        NodeKind::Element(e) => {
            if e.is_block_element() {
                DisplayType::Block
            } else {
                DisplayType::Inline
            }
        }
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
