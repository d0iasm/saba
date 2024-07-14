//! This is used from UI component.

use crate::renderer::layout::computed_style::ComputedStyle;
use crate::renderer::layout::layout_point::LayoutPoint;
use crate::renderer::layout::layout_size::LayoutSize;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayItem {
    Rect {
        style: ComputedStyle,
        layout_point: LayoutPoint,
        layout_size: LayoutSize,
    },
    // TODO: remove Link and merge it into Text by adding `text-decoration: underline;`.
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
    Img {
        src: String,
        style: ComputedStyle,
        layout_point: LayoutPoint,
    },
}

impl DisplayItem {
    pub fn is_rect(&self) -> bool {
        matches!(
            self,
            DisplayItem::Rect {
                style: _,
                layout_point: _,
                layout_size: _,
            }
        )
    }

    pub fn is_link(&self) -> bool {
        matches!(
            self,
            DisplayItem::Link {
                text: _,
                destination: _,
                style: _,
                layout_point: _,
            }
        )
    }

    pub fn is_text(&self) -> bool {
        matches!(
            self,
            DisplayItem::Text {
                text: _,
                style: _,
                layout_point: _,
            }
        )
    }
}
