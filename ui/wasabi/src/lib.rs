#![no_std]

extern crate alloc;

use alloc::rc::Weak;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use noli::{window::StringSize, window::Window};
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

//static WINDOW_WIDTH: i64 = 1024;
//static WINDOW_HEIGHT: i64 = 768;
static WINDOW_WIDTH: i64 = 600;
static WINDOW_HEIGHT: i64 = 400;
static WINDOW_PADDING: i64 = 5;

static TOOLBAR_HEIGHT: i64 = 26;
static ADDRESSBAR_HEIGHT: i64 = 20;

static CHAR_WIDTH: i64 = 8;
static CHAR_HEIGHT: i64 = 16;

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
            window: Window::new(
                "toybr".to_string(),
                WHITE,
                0,
                0,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .unwrap(),
            position: (WINDOW_PADDING, TOOLBAR_HEIGHT + WINDOW_PADDING),
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

        let _ = self.start_navigation(handle_url, "http://example.com".to_string());

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
                            /*underline=*/ true,
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
                                /*underline=*/ false,
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
