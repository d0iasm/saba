//! https://developer.mozilla.org/en-US/docs/Web/CSS/computed_value
//! https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/style/computed_style.h
//! https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/Cascade_and_inheritance

use crate::renderer::html::parser::{ElementKind, Node, NodeKind};
use crate::renderer::layout::color::*;
use alloc::rc::Rc;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    background_color: Option<Color>,
    color: Option<Color>,
    display: Option<DisplayType>,
    font_size: Option<FontSize>,
    height: Option<f64>,
    margin: Option<BoxInfo>,
    padding: Option<BoxInfo>,
    text_decoration: Option<TextDecoration>,
    white_space: Option<WhiteSpace>,
    width: Option<f64>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self::new()
    }
}

/// The value of a CSS property is converted like:
/// a declared value
/// -> a cascaded value
/// -> a specified value
/// -> a computed value
/// -> a used value
/// -> an actual value
/// https://www.w3.org/TR/css-cascade-4/#value-stages
///
/// "The computed value is the value that is transferred from parent to child during inheritance."
/// https://www.w3.org/TR/css-cascade-4/#computed
///
/// https://www.w3.org/TR/css-cascade-4/#stages-examples
///
/// This ComputedStyle contains the information of all computed values that this browser supports.
/// Used values/actual values are yielded just before it's displayed on UI.
impl ComputedStyle {
    pub fn new() -> Self {
        // It may be better to handle cascading, defaulting and inheritance here.
        Self {
            background_color: None,
            color: None,
            display: None,
            font_size: None,
            height: None,
            margin: None,
            padding: None,
            text_decoration: None,
            white_space: None,
            width: None,
        }
    }

    /// https://www.w3.org/TR/css-cascade-4/#defaulting
    /// If there is no cascading value, use the default value.
    pub fn defaulting(&mut self, node: &Rc<RefCell<Node>>) {
        if self.background_color.is_none() {
            self.background_color = Some(Color::white());
        }
        if self.color.is_none() {
            self.color = Some(Color::black());
        }
        if self.display.is_none() {
            self.display = Some(DisplayType::default(node));
        }
        if self.font_size.is_none() {
            self.font_size = Some(FontSize::default(node));
        }
        if self.height.is_none() {
            // check the default value for height
            self.height = Some(0.0);
        }
        if self.margin.is_none() {
            // check the default value for margin
            self.margin = Some(BoxInfo::new(0.0, 0.0, 0.0, 0.0));
        }
        if self.padding.is_none() {
            // check the default value for padding
            self.padding = Some(BoxInfo::new(0.0, 0.0, 0.0, 0.0));
        }
        if self.text_decoration.is_none() {
            self.text_decoration = Some(TextDecoration::default(node));
        }
        if self.white_space.is_none() {
            self.white_space = Some(WhiteSpace::default(node));
        }
        if self.width.is_none() {
            // check the default value for width
            self.width = Some(0.0);
        }
    }

    /// https://www.w3.org/TR/css-cascade-4/#inheriting
    /// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/css/resolver/style_resolver.h;drc=48340c1e35efad5fb0253025dcc36b3a9573e258;bpv=1;bpt=1;l=234
    pub fn inherit(&mut self, parent_style: &ComputedStyle) {
        self.background_color = Some(parent_style.background_color());
        self.color = Some(parent_style.color());
        self.display = Some(parent_style.display());
        self.font_size = Some(parent_style.font_size());
        self.height = Some(parent_style.height());
        self.margin = Some(parent_style.margin());
        self.padding = Some(parent_style.padding());
        self.text_decoration = Some(parent_style.text_decoration());
        self.white_space = Some(parent_style.white_space());
        self.width = Some(parent_style.width());
        /*
        if self.background_color.is_none() {
            self.background_color = Some(parent_style.background_color().clone());
        }
        if self.color.is_none() {
            self.color = Some(parent_style.color().clone());
        }
        if self.display.is_none() {
            self.display = Some(parent_style.display().clone());
        }
        if self.font_size.is_none() {
            self.font_size = Some(parent_style.font_size().clone());
        }
        if self.height.is_none() {
            self.height = Some(parent_style.height().clone());
        }
        if self.margin.is_none() {
            self.margin = Some(parent_style.margin().clone());
        }
        if self.padding.is_none() {
            self.padding = Some(parent_style.padding().clone());
        }
        if self.text_decoration.is_none() {
            self.text_decoration = Some(parent_style.text_decoration().clone());
        }
        if self.white_space.is_none() {
            self.white_space = Some(parent_style.white_space().clone());
        }
        if self.width.is_none() {
            self.width = Some(parent_style.width().clone());
        }
        */
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = Some(color);
    }

    pub fn background_color(&self) -> Color {
        self.background_color
            .clone()
            .expect("failed to access CSS property: background_color")
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = Some(color);
    }

    pub fn color(&self) -> Color {
        self.color
            .clone()
            .expect("failed to access CSS property: color")
    }

    pub fn set_height(&mut self, height: f64) {
        self.height = Some(height);
    }

    pub fn height(&self) -> f64 {
        self.height.expect("failed to access CSS property: height")
    }

    pub fn display(&self) -> DisplayType {
        self.display
            .expect("failed to access CSS property: display")
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    pub fn width(&self) -> f64 {
        self.width.expect("failed to access CSS property: width")
    }

    pub fn set_margin(&mut self, margin: BoxInfo) {
        self.margin = Some(margin);
    }

    pub fn margin(&self) -> BoxInfo {
        self.margin.expect("failed to access CSS property: margin")
    }

    pub fn set_padding(&mut self, padding: BoxInfo) {
        self.padding = Some(padding);
    }

    pub fn padding(&self) -> BoxInfo {
        self.padding
            .expect("failed to access CSS property: padding")
    }

    pub fn font_size(&self) -> FontSize {
        self.font_size
            .expect("failed to access CSS property: font_size")
    }

    pub fn white_space(&self) -> WhiteSpace {
        self.white_space
            .expect("failed to access CSS property: white_space")
    }

    pub fn text_decoration(&self) -> TextDecoration {
        self.text_decoration
            .expect("failed to access CSS property: text_decoration")
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

impl DisplayType {
    fn default(node: &Rc<RefCell<Node>>) -> Self {
        match &node.borrow().kind() {
            NodeKind::Document => DisplayType::Block,
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
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
/// https://docs.gtk.org/Pango/pango_markup.html align with pango markup syntax
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FontSize {
    Medium,
    XLarge,
    XXLarge,
}

impl FontSize {
    fn default(node: &Rc<RefCell<Node>>) -> Self {
        match &node.borrow().kind() {
            NodeKind::Element(element) => match element.kind() {
                ElementKind::H1 => FontSize::XXLarge,
                ElementKind::H2 => FontSize::XLarge,
                _ => FontSize::Medium,
            },
            _ => FontSize::Medium,
        }
    }
}

/// https://w3c.github.io/csswg-drafts/css-text-decor/#text-decoration-property
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TextDecoration {
    None,
    Underline,
}

impl TextDecoration {
    fn default(node: &Rc<RefCell<Node>>) -> Self {
        match &node.borrow().kind() {
            NodeKind::Element(element) => match element.kind() {
                ElementKind::A => TextDecoration::Underline,
                _ => TextDecoration::None,
            },
            _ => TextDecoration::None,
        }
    }
}

/// https://w3c.github.io/csswg-drafts/css-text/#white-space-property
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WhiteSpace {
    Normal,
    Pre,
}

impl WhiteSpace {
    fn default(node: &Rc<RefCell<Node>>) -> Self {
        match &node.borrow().kind() {
            NodeKind::Element(element) => match element.kind() {
                ElementKind::P => WhiteSpace::Normal,
                ElementKind::Pre => WhiteSpace::Pre,
                _ => WhiteSpace::Normal,
            },
            _ => WhiteSpace::Normal,
        }
    }
}
