//! This is the size of a rect.
//! https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_box.h;drc=48340c1e35efad5fb0253025dcc36b3a9573e258;bpv=1;bpt=1;l=2404

/// The size of this layout object.
/// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/geometry/physical_size.h
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LayoutSize {
    width: f64,
    height: f64,
}

impl LayoutSize {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: f64) {
        self.height = height;
    }
}
