//! This is used from UI component.

use crate::renderer::layout::computed_style::{ComputedStyle, LayoutPosition};

#[derive(Debug, Clone)]
pub enum DisplayItem {
    Rect {
        style: ComputedStyle,
        position: LayoutPosition,
    },
    Link {
        text: String,
        destination: String,
        style: ComputedStyle,
        position: LayoutPosition,
    },
    Text {
        text: String,
        style: ComputedStyle,
        position: LayoutPosition,
    },
}

impl DisplayItem {
    pub fn is_rect(&self) -> bool {
        match self {
            DisplayItem::Rect {
                style: _,
                position: _,
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
                position: _,
            } => true,
            _ => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            DisplayItem::Text {
                text: _,
                style: _,
                position: _,
            } => true,
            _ => false,
        }
    }
}
