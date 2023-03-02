use graphics::rectangle::Rectangle;
use piston_window::*;

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
}

pub fn draw_line(window: &mut Window) {
    while let Some(e) = window.canvas.next() {
        println!("-------------------------- {:?}", e);
        window.canvas.draw_2d(&e, |c, g, _| {
            clear([1.0; 4], g);
            let orange = [1.0, 0.5, 0.0, 1.0];
            line(
                orange,
                5.0,
                [320.0 - 1.0 * 15.0, 20.0, 380.0 - 1.0 * 15.0, 80.0],
                c.transform,
                g,
            );
            let magenta = [1.0, 0.0, 0.5, 1.0];
            polygon(
                magenta,
                &[[420.0, 20.0], [480.0, 20.0], [480.0 - 1.0 * 15.0, 80.0]],
                c.transform,
                g,
            );
        });
    }
}

pub fn draw_rect(window: &mut Window) {
    while let Some(e) = window.canvas.next() {
        println!("!!!!!!!!!!!!!!!!!!!!!!!!!!1 {:?}", e);
        window.canvas.draw_2d(&e, |c, g, _| {
            clear([1.0; 4], g);
            let black = [0.0, 0.0, 0.0, 1.0];
            let red = [1.0, 0.0, 0.0, 1.0];
            let rect = math::margin_rectangle([20.0, 20.0, 60.0, 60.0], 0.0);
            rectangle(red, rect, c.transform, g);
            Rectangle::new_border(black, 2.0).draw(rect, &c.draw_state, c.transform, g);
        });
    }
}

pub fn print(text: &str) {
    print!("{}", text);
}

pub fn println(text: &str) {
    println!("{}", text);
}
