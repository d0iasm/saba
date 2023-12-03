#![no_std]

extern crate alloc;

use alloc::rc::Weak;
use alloc::string::String;
use core::cell::RefCell;
use noli::*;
use toybr_core::{browser::Browser, error::Error, http::HttpResponse, ui::UiObject};

static WHITE: u32 = 0xffffff;
static RED: u32 = 0xff0000;
static GREEN: u32 = 0x00ff00;
static BLUE: u32 = 0x0000ff;
static DARKBLUE: u32 = 0x00008b;
static GREY: u32 = 0x808080;
static DARKGREY: u32 = 0x5a5a5a;
static BLACK: u32 = 0x000000;

//static WIDTH: i64 = 1024;
//static HEIGHT: i64 = 768;
static WIDTH: i64 = 500;
static HEIGHT: i64 = 300;

static BUTTON_SIZE: i64 = 14;

#[derive(Clone, Debug)]
pub struct WasabiUI {
    browser: Weak<RefCell<Browser<Self>>>,
    input_url: String,
}

impl UiObject for WasabiUI {
    fn new() -> Self {
        Self {
            browser: Weak::new(),
            input_url: String::new(),
        }
    }

    fn console_debug(&mut self, log: String) {}
    fn console_warning(&mut self, log: String) {}
    fn console_error(&mut self, log: String) {}
    fn start(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        self.setup();

        Ok(())
    }
}

impl WasabiUI {
    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser<WasabiUI>>>) {
        self.browser = browser;
    }

    pub fn browser(&self) -> Weak<RefCell<Browser<Self>>> {
        self.browser.clone()
    }

    fn setup(&self) {
        fill_rect(WHITE, 10, 10, WIDTH, HEIGHT).unwrap();

        self.toolbar();
    }

    fn toolbar(&self) {
        fill_rect(DARKBLUE, 10, 10, WIDTH, 20).unwrap();

        fill_rect(DARKGREY, WIDTH - 10, 13, BUTTON_SIZE, BUTTON_SIZE).unwrap();
        draw_line(WHITE, WIDTH - 10, 13, WIDTH - 10, 13 + BUTTON_SIZE);
        draw_line(WHITE, WIDTH - 10, 13, WIDTH - 10 + BUTTON_SIZE, 13);
        draw_line(
            BLACK,
            WIDTH - 10,
            13,
            WIDTH - 10 + BUTTON_SIZE,
            13 + BUTTON_SIZE,
        )
        .unwrap();
        draw_line(
            BLACK,
            WIDTH - 10 + BUTTON_SIZE,
            13,
            WIDTH - 10,
            13 + BUTTON_SIZE,
        )
        .unwrap();
    }
}
