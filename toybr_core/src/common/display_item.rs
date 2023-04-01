//! This is used from UI component.

use crate::renderer::layout::computed_style::ComputedStyle;
use crate::renderer::layout::layout_point::LayoutPoint;

#[derive(Debug, Clone)]
pub enum DisplayItem {
    Rect {
        style: ComputedStyle,
        layout_point: LayoutPoint,
    },
    Link {
        text: String,
        destination: String,
        style: ComputedStyle,
        layout_point: LayoutPoint,
    },
    Text {
        text: String,
        style: ComputedStyle,
        layout_point: LayoutPoint,
    },
}

impl DisplayItem {
    pub fn is_rect(&self) -> bool {
        match self {
            DisplayItem::Rect {
                style: _,
                layout_point: _,
            } => true,
            _ => false,
        }
    }

    pub fn is_link(&self) -> bool {
        match self {
            DisplayItem::Link {
                text: _,
                destination: _,
                style: _,
                layout_point: _,
            } => true,
            _ => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            DisplayItem::Text {
                text: _,
                style: _,
                layout_point: _,
            } => true,
            _ => false,
        }
    }
}
