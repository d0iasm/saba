//! The CSS border box rect for this box.
//! https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/layout/layout_box.h;drc=48340c1e35efad5fb0253025dcc36b3a9573e258;bpv=1;bpt=1;l=2401

/// The start point (x, y) of the layout object.
/// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/platform/geometry/layout_point.h
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LayoutPoint {
    x: f64,
    y: f64,
}

impl LayoutPoint {
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
