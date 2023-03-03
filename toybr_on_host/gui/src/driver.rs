use crate::artifact;
use skia_safe::{Canvas, Surface};
use std::{fmt::Display, path::Path, str::FromStr};

pub struct Cpu;

pub trait DrawingDriver {
    const DRIVER: Driver;

    fn new() -> Self;

    fn draw_image(&mut self, size: (i32, i32), path: &Path, name: &str, func: impl Fn(&mut Canvas));

    fn draw_image_256(&mut self, path: &Path, name: &str, func: impl Fn(&mut Canvas)) {
        self.draw_image((256, 256), path, name, func)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Driver {
    Cpu,
    Pdf,
    Svg,
}

impl FromStr for Driver {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Driver::*;
        Ok(match s {
            "cpu" => Cpu,
            "pdf" => Pdf,
            "svg" => Svg,
            _ => return Err("Unknown driver"),
        })
    }
}

impl Display for Driver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Driver::*;
        let name = match self {
            Cpu => "cpu",
            Pdf => "pdf",
            Svg => "svg",
        };
        f.write_str(name)
    }
}

impl DrawingDriver for Cpu {
    const DRIVER: Driver = Driver::Cpu;

    fn new() -> Self {
        Self
    }

    fn draw_image(
        &mut self,
        (width, height): (i32, i32),
        path: &Path,
        name: &str,
        func: impl Fn(&mut Canvas),
    ) {
        let mut surface = Surface::new_raster_n32_premul((width * 2, height * 2)).unwrap();
        artifact::draw_image_on_surface(&mut surface, path, name, func);
    }
}
