//use crate::{resources, DrawingDriver};
use crate::driver::DrawingDriver;
use skia_safe::{
    paint, scalar, BlendMode, Canvas, Color, Font, Paint, Path, RRect, Rect, TextBlob, Typeface,
};
use std::path;

pub fn draw(driver: &mut impl DrawingDriver, path: &path::Path) {
    let path = path.join("SkCanvas-Overview");
    driver.draw_image_256(&path, "heptagram", draw_heptagram);
    driver.draw_image_256(&path, "rotated-rectangle", draw_rotated_rectangle);
    //driver.draw_image_256(&path, "hello-skia", draw_hello_skia);
}

fn draw_heptagram(canvas: &mut Canvas) {
    const SCALE: scalar = 256.0;
    const R: scalar = 0.45 * SCALE;
    #[allow(clippy::excessive_precision)]
    const TAU: scalar = std::f32::consts::TAU;
    let mut path = Path::default();
    path.move_to((R, 0.0));
    for i in 1..7 {
        let theta = 3.0 * (i as scalar) * TAU / 7.0;
        path.line_to((R * scalar::cos(theta), R * scalar::sin(theta)));
    }

    path.close();
    let mut p = Paint::default();
    p.set_anti_alias(true);
    canvas
        .clear(Color::WHITE)
        .translate((0.5 * SCALE, 0.5 * SCALE))
        .draw_path(&path, &p);
}

fn draw_rotated_rectangle(canvas: &mut Canvas) {
    canvas.save();
    canvas.translate((128.0, 128.0)).rotate(45.0, None);
    let rect = Rect::from_point_and_size((-90.5, -90.5), (181.0, 181.0));
    let mut paint = Paint::default();
    paint.set_color(Color::BLUE);
    canvas.draw_rect(rect, &paint);
    canvas.restore();
}

/*
fn draw_hello_skia(canvas: &mut Canvas) {
    let image = resources::color_wheel();

    canvas.draw_color(Color::WHITE, BlendMode::default());

    let mut paint = Paint::default();
    paint
        .set_style(paint::Style::Stroke)
        .set_stroke_width(4.0)
        .set_color(Color::RED);

    let mut rect = Rect::from_point_and_size((50.0, 50.0), (40.0, 60.0));
    canvas.draw_rect(rect, &paint);

    let oval = RRect::new_oval(rect).with_offset((40.0, 60.0));
    paint.set_color(Color::BLUE);
    canvas.draw_rrect(oval, &paint);

    paint.set_color(Color::CYAN);
    canvas.draw_circle((180.0, 50.0), 25.0, &paint);

    rect = rect.with_offset((80.0, 0.0));
    paint.set_color(Color::YELLOW);
    canvas.draw_round_rect(rect, 10.0, 10.0, &paint);

    let mut path = Path::default();
    path.cubic_to((768.0, 0.0), (-512.0, 256.0), (256.0, 256.0));
    paint.set_color(Color::GREEN);
    canvas.draw_path(&path, &paint);

    canvas.draw_image(&image, (128.0, 128.0), Some(&paint));

    let rect2 = Rect::from_point_and_size((0.0, 0.0), (40.0, 60.0));
    canvas.draw_image_rect(&image, None, rect2, &paint);

    let paint2 = Paint::default();

    let text = TextBlob::from_str(
        "Hello, Skia!",
        &Font::from_typeface(&Typeface::default(), 18.0),
    )
    .unwrap();
    canvas.draw_text_blob(&text, (50, 25), &paint2);
}
*/

/*
use renderer::layout::render_tree::RenderTree;

pub struct Window {
    pub canvas: PistonWindow,
}

impl Window {
    pub fn new() -> Self {
        let mut window: PistonWindow = WindowSettings::new("toybr", (1024, 768))
            .exit_on_esc(true)
            .graphics_api(OpenGL::V3_2)
            .build()
            .unwrap();
        window.set_lazy(true);

        Self { canvas: window }
    }

    pub fn start(&mut self, handle_input: fn(String) -> RenderTree) {
        handle_input("http://example.com".to_string());
        while let Some(e) = self.canvas.next() {
            //println(&format!("-------------------------- {:?}", e));
            self.canvas.draw_2d(&e, |c, g, _| {
                clear([1.0; 4], g);
                let orange = [1.0, 0.5, 0.0, 1.0];

                line(
                    orange,
                    5.0,
                    [320.0 - 1.0 * 15.0, 20.0, 380.0 - 1.0 * 15.0, 80.0],
                    c.transform,
                    g,
                );
            });
        }
    }

    fn draw_text(&mut self) {}
    fn draw_rect(&mut self) {}
    fn draw_line(&mut self) {}
}
*/
