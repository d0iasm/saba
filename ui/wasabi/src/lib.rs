#![no_std]

extern crate alloc;

use alloc::rc::Weak;
use alloc::string::String;
use alloc::string::ToString;
use core::cell::RefCell;
use noli::{window::StringSize, window::Window, *};
use toybr_core::{
    browser::Browser, display_item::DisplayItem, error::Error, http::HttpResponse,
    renderer::layout::computed_style::FontSize, ui::UiObject,
};

static WHITE: u32 = 0xffffff;
static RED: u32 = 0xff0000;
static GREEN: u32 = 0x00ff00;
static BLUE: u32 = 0x0000ff;
static DARKBLUE: u32 = 0x00008b;
static LIGHTGREY: u32 = 0xd3d3d3;
static GREY: u32 = 0x808080;
static DARKGREY: u32 = 0x5a5a5a;
static BLACK: u32 = 0x000000;

//static WIDTH: i64 = 1024;
//static HEIGHT: i64 = 768;
static WIDTH: i64 = 600;
static HEIGHT: i64 = 400;

static TOOLBAR_HEIGHT: i64 = 26;
static ADDRESSBAR_HEIGHT: i64 = 20;

#[derive(Clone, Debug)]
pub struct WasabiUI {
    browser: Weak<RefCell<Browser<Self>>>,
    input_url: String,
    window: Window,
    // The (x, y) position to render a next display item.
    position: (i64, i64),
}

impl UiObject for WasabiUI {
    fn new() -> Self {
        Self {
            browser: Weak::new(),
            input_url: String::new(),
            window: Window::new("toybr".to_string(), WHITE, 0, 0, WIDTH, HEIGHT).unwrap(),
            position: (5, TOOLBAR_HEIGHT + 5),
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

        self.start_navigation(handle_url, "http://example.com".to_string());

        self.update_ui();

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

    fn setup(&self) -> Result<(), Error> {
        self.toolbar()?;

        Ok(())
    }

    fn toolbar(&self) -> Result<(), Error> {
        if self
            .window
            .fill_rect(LIGHTGREY, 0, 0, WIDTH, TOOLBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        if self
            .window
            .draw_line(GREY, 0, TOOLBAR_HEIGHT, WIDTH, TOOLBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }
        if self
            .window
            .draw_line(DARKGREY, 0, TOOLBAR_HEIGHT + 1, WIDTH, TOOLBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        if self
            .window
            .draw_string(BLACK, 5, 5, "Address:", StringSize::Medium)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        // address bar
        if self
            .window
            .fill_rect(WHITE, 70, 2, WIDTH - 74, 2 + ADDRESSBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        // shadow for address bar
        if self.window.draw_line(GREY, 70, 2, WIDTH - 4, 2).is_err() {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }
        if self
            .window
            .draw_line(GREY, 70, 2, 70, 2 + ADDRESSBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }
        if self.window.draw_line(BLACK, 71, 3, WIDTH - 5, 3).is_err() {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        if self
            .window
            .draw_line(GREY, 71, 3, 71, 1 + ADDRESSBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        /*
        // high light for address bar
        if self
            .window
            .draw_line(
                LIGHTGREY,
                71,
                2 + ADDRESSBAR_HEIGHT,
                WIDTH - 5,
                2 + ADDRESSBAR_HEIGHT,
            )
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }
        if self
            .window
            .draw_line(LIGHTGREY, WIDTH - 5, 2, WIDTH - 5, 2 + ADDRESSBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }
        */

        Ok(())
    }

    fn start_navigation(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
        destination: String,
    ) -> Result<(), Error> {
        match handle_url(destination) {
            Ok(response) => {
                let page = match self.browser().upgrade() {
                    Some(browser) => {
                        // clean up Browser struct
                        {
                            browser.borrow_mut().clear_display_items();
                        }
                        {
                            browser.borrow_mut().clear_logs();
                        }

                        browser.borrow().page()
                    }
                    None => {
                        return Err(Error::Other("associated browser is not found".to_string()))
                    }
                };

                page.borrow_mut().receive_response(response);
            }
            Err(e) => {
                //self.console_error(format!("{:?}", e));
                return Err(e);
            }
        }
        Ok(())
    }

    fn update_ui(&mut self) {
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };
        let display_items = browser.borrow().display_items();

        for item in display_items {
            match item {
                DisplayItem::Rect {
                    style: _,
                    layout_point: _,
                    layout_size: _,
                } => {}
                DisplayItem::Link {
                    text,
                    destination,
                    style,
                    layout_point: _,
                } => {
                    self.window
                        .draw_string(
                            style.color().code_u32(),
                            self.position.0,
                            self.position.1,
                            &text,
                            StringSize::Medium,
                        )
                        .unwrap();
                    self.position.1 += 20;
                }
                DisplayItem::Text {
                    text,
                    style,
                    layout_point: _,
                } => {
                    let string_size = convert_font_size(style.font_size());
                    for line in text.split("\n") {
                        self.window
                            .draw_string(
                                style.color().code_u32(),
                                self.position.0,
                                self.position.1,
                                &line.trim(),
                                string_size.clone(),
                            )
                            .unwrap();

                        match string_size {
                            StringSize::Medium => self.position.1 += 20,
                            StringSize::Large => self.position.1 += 40,
                            StringSize::XLarge => self.position.1 += 60,
                        }
                    }
                }
            }
        }
    }
}

fn convert_font_size(size: FontSize) -> StringSize {
    match size {
        FontSize::Medium => StringSize::Medium,
        FontSize::XLarge => StringSize::Large,
        FontSize::XXLarge => StringSize::XLarge,
    }
}
