use crate::*;
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

    pub fn start(&mut self) {
        while let Some(e) = self.canvas.next() {
            println(&format!("-------------------------- {:?}", e));
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
}
