use noli::bitmap::bitmap_draw_rect;
use noli::rect::Rect;
use noli::sheet::Sheet;

#[derive(Debug, Eq, PartialEq)]
pub struct Cursor {
    sheet: Sheet,
}

impl Cursor {
    pub fn new() -> Self {
        let mut sheet = Sheet::new(Rect::new(0, 0, 10, 10).unwrap());
        let bitmap = sheet.bitmap();
        bitmap_draw_rect(bitmap, 0xff0000, 0, 0, 10, 10).expect("failed to draw a cursor");

        Self { sheet }
    }

    pub fn rect(&self) -> Rect {
        self.sheet.rect()
    }

    pub fn set_position(&mut self, x: i64, y: i64) {
        self.sheet.set_position(x, y);
    }

    pub fn flush(&mut self) {
        self.sheet.flush();
    }
}
