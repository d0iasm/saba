#![no_std]

extern crate alloc;

use alloc::rc::Weak;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use embedded_graphics::primitives::Circle;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{
    image::{Image, ImageRaw, ImageRawBE},
    pixelcolor::Rgb565,
    prelude::*,
};
use noli::{print, window::StringSize, window::Window};
use saba_core::{
    browser::Browser,
    display_item::DisplayItem,
    error::Error,
    http::HttpResponse,
    renderer::layout::computed_style::{FontSize, TextDecoration},
};

static WHITE: u32 = 0xffffff;
static _RED: u32 = 0xff0000;
static _GREEN: u32 = 0x00ff00;
static _BLUE: u32 = 0x0000ff;
static _DARKBLUE: u32 = 0x00008b;
static LIGHTGREY: u32 = 0xd3d3d3;
static GREY: u32 = 0x808080;
static DARKGREY: u32 = 0x5a5a5a;
static BLACK: u32 = 0x000000;

//static WINDOW_WIDTH: i64 = 1024;
//static WINDOW_HEIGHT: i64 = 768;
static WINDOW_WIDTH: i64 = 600;
static WINDOW_HEIGHT: i64 = 400;
static WINDOW_PADDING: i64 = 5;

// defined in noli/src/window.rs
static TITLE_BAR_HEIGHT: i64 = 24;

static CONTENT_AREA_WIDTH: i64 = WINDOW_WIDTH;
static CONTENT_AREA_HEIGHT: i64 = WINDOW_HEIGHT - TITLE_BAR_HEIGHT - TOOLBAR_HEIGHT;

static TOOLBAR_HEIGHT: i64 = 26;
static ADDRESSBAR_HEIGHT: i64 = 20;

static CHAR_WIDTH: i64 = 8;
static _CHAR_HEIGHT: i64 = 16;

#[derive(Clone, Debug)]
pub struct WasabiUI {
    browser: Weak<RefCell<Browser>>,
    input_url: String,
    window: Window,
    // The (x, y) position to render a next display item.
    position: (i64, i64),
}

impl WasabiUI {
    pub fn new() -> Self {
        Self {
            browser: Weak::new(),
            input_url: String::new(),
            window: Window::new(
                "SaBA".to_string(),
                WHITE,
                30,
                50,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .unwrap(),
            position: (WINDOW_PADDING, TOOLBAR_HEIGHT + WINDOW_PADDING),
        }
    }

    pub fn start(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        self.setup()?;

        // never return unless a user quits the app.
        self.run_app(handle_url)?;

        Ok(())
    }

    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser>>) {
        self.browser = browser;
    }

    pub fn browser(&self) -> Weak<RefCell<Browser>> {
        self.browser.clone()
    }

    fn setup(&self) -> Result<(), Error> {
        self.setup_toolbar()?;

        Ok(())
    }

    fn setup_toolbar(&self) -> Result<(), Error> {
        if self
            .window
            .fill_rect(LIGHTGREY, 0, 0, WINDOW_WIDTH, TOOLBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        if self
            .window
            .draw_line(GREY, 0, TOOLBAR_HEIGHT, WINDOW_WIDTH, TOOLBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }
        if self
            .window
            .draw_line(
                DARKGREY,
                0,
                TOOLBAR_HEIGHT + 1,
                WINDOW_WIDTH,
                TOOLBAR_HEIGHT,
            )
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        if self
            .window
            .draw_string(
                BLACK,
                5,
                5,
                "Address:",
                StringSize::Medium,
                /*underline=*/ false,
            )
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        // address bar
        if self
            .window
            .fill_rect(WHITE, 70, 2, WINDOW_WIDTH - 74, 2 + ADDRESSBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        // shadow for address bar
        if self
            .window
            .draw_line(GREY, 70, 2, WINDOW_WIDTH - 4, 2)
            .is_err()
        {
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
        if self
            .window
            .draw_line(BLACK, 71, 3, WINDOW_WIDTH - 5, 3)
            .is_err()
        {
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
                WINDOW_WIDTH - 5,
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
            .draw_line(LIGHTGREY, WINDOW_WIDTH - 5, 2, WINDOW_WIDTH - 5, 2 + ADDRESSBAR_HEIGHT)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }
        */

        Ok(())
    }

    fn run_app(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        loop {
            if let Some(c) = noli::sys::read_key() {
                if c == 0xA as char || c == '\n' {
                    // enter key
                    self.clear_content_area()?;

                    let _ = self.start_navigation(handle_url, "http://example.com".to_string());
                    self.update_ui()?;

                    self.clear_address_bar()?;
                    self.input_url = String::new();
                } else if c == 0x7F as char {
                    // delete key
                    self.input_url.pop();
                    self.update_address_bar()?;
                } else {
                    self.input_url.push(c);
                    self.update_address_bar()?;
                }
            }
        }
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
                return Err(e);
            }
        }
        Ok(())
    }

    fn update_ui(&mut self) -> Result<(), Error> {
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => {
                return Err(Error::Other(
                    "failed to obtain a browser object".to_string(),
                ))
            }
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
                    destination: _,
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
                            style.text_decoration() == TextDecoration::Underline,
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
                    let char_width = match string_size {
                        StringSize::Medium => CHAR_WIDTH,
                        StringSize::Large => CHAR_WIDTH * 2,
                        StringSize::XLarge => CHAR_WIDTH * 3,
                    };
                    // replace new lines to white spaces and replace sequential multiple white
                    // spaces with one white space.
                    let plain_text = text
                        .replace("\n", " ")
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let lines = split_text(plain_text, char_width);

                    for line in lines {
                        self.window
                            .draw_string(
                                style.color().code_u32(),
                                self.position.0,
                                self.position.1,
                                &line,
                                string_size.clone(),
                                style.text_decoration() == TextDecoration::Underline,
                            )
                            .unwrap();

                        match string_size {
                            StringSize::Medium => self.position.1 += 20,
                            StringSize::Large => self.position.1 += 40,
                            StringSize::XLarge => self.position.1 += 60,
                        }
                    }
                }
                DisplayItem::Img {
                    src,
                    style: _,
                    layout_point: _,
                } => {
                    print!("DisplayItem::Img src: {}\n", src);

                    //let data = include_bytes!("/Users/asami/Projects/saba/test.png");
                    const data: &[u8] = &[
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                        0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0,
                    ];
                    let img: ImageRawBE<Rgb565> = ImageRaw::new(data, 10);
                    let image = Image::new(&img, Point::new(100, 100));
                    //print!("image: {:#?}\n", image);

                    /*
                    let circle = Circle::new(Point::new(100, 100), 100)
                        .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 5));
                    circle.draw(&mut self.window);
                    */

                    if image.draw(&mut self.window).is_err() {
                        return Err(Error::Other("failed to draw an image".to_string()));
                    }
                }
            }
        }

        for log in browser.borrow().logs() {
            print!("{}\n", log.to_string());
        }

        Ok(())
    }

    fn update_address_bar(&self) -> Result<(), Error> {
        // draw URL string
        if self
            .window
            .draw_string(
                BLACK,
                74,
                6,
                &self.input_url,
                StringSize::Medium,
                /*underline=*/ false,
            )
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        Ok(())
    }

    fn clear_address_bar(&mut self) -> Result<(), Error> {
        // clear address bar
        if self
            .window
            .fill_rect(WHITE, 72, 4, WINDOW_WIDTH - 76, ADDRESSBAR_HEIGHT - 2)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        Ok(())
    }

    fn clear_content_area(&mut self) -> Result<(), Error> {
        self.position = (WINDOW_PADDING, TOOLBAR_HEIGHT + WINDOW_PADDING);

        // fill out the content area with white box
        if self
            .window
            .fill_rect(
                WHITE,
                0,
                TOOLBAR_HEIGHT + 2,
                CONTENT_AREA_WIDTH,
                CONTENT_AREA_HEIGHT - 2,
            )
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to initialize a toolbar".to_string(),
            ));
        }

        Ok(())
    }
}

/// Converts FontSize, defined in renderer::layout::computed_style::FontSize, to StringSize to make
/// it compatible with noli library.
fn convert_font_size(size: FontSize) -> StringSize {
    match size {
        FontSize::Medium => StringSize::Medium,
        FontSize::XLarge => StringSize::Large,
        FontSize::XXLarge => StringSize::XLarge,
    }
}

/// This is used when { word-break: normal; } in CSS.
/// https://drafts.csswg.org/css-text/#word-break-property
fn find_index_for_line_break(line: String, max_index: usize) -> usize {
    for i in (0..max_index).rev() {
        if line.chars().collect::<Vec<char>>()[i] == ' ' {
            return i;
        }
    }
    max_index
}

/// https://drafts.csswg.org/css-text/#word-break-property
fn split_text(line: String, char_width: i64) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    if line.len() as i64 * char_width > (WINDOW_WIDTH + WINDOW_PADDING) {
        let s = line.split_at(find_index_for_line_break(
            line.clone(),
            ((WINDOW_WIDTH + WINDOW_PADDING) / char_width) as usize,
        ));
        result.push(s.0.to_string());
        result.extend(split_text(s.1.trim().to_string(), char_width))
    } else {
        result.push(line);
    }
    result
}
