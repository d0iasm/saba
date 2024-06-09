#![no_std]

extern crate alloc;

use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::include_bytes;
use embedded_graphics::{image::Image, pixelcolor::Rgb888, prelude::*};
use noli::prelude::SystemApi;
use noli::print;
use noli::println;
use noli::sys::api::MouseEvent;
use noli::sys::wasabi::Api;
use noli::window::StringSize;
use noli::window::Window;
use saba_core::{
    browser::Browser,
    constants::*,
    display_item::DisplayItem,
    error::Error,
    http::HttpResponse,
    renderer::layout::computed_style::{FontSize, TextDecoration},
};
use tinybmp::{Bmp, RawBmp};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone, Debug)]
pub struct WasabiUI {
    browser: Rc<RefCell<Browser>>,
    input_url: String,
    input_mode: InputMode,
    window: Window,
    // The (x, y) position to render a next display item.
    position: (i64, i64),
}

impl WasabiUI {
    pub fn new(browser: Rc<RefCell<Browser>>) -> Self {
        Self {
            browser,
            input_url: String::new(),
            input_mode: InputMode::Normal,
            window: Window::new(
                "SaBA".to_string(),
                WHITE,
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS,
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

    pub fn browser(&self) -> Rc<RefCell<Browser>> {
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

    fn handle_key_input(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        match self.input_mode {
            InputMode::Normal => {
                // ignore a key when input_mode is Normal.
                let _ = Api::read_key();
            }
            InputMode::Editing => {
                if let Some(c) = Api::read_key() {
                    if c == 0xA as char || c == '\n' {
                        // enter key
                        self.clear_content_area()?;

                        let _ = self.start_navigation_from_toolbar(
                            handle_url,
                            "http://example.com".to_string(),
                        );
                        self.update_ui()?;

                        self.input_mode = InputMode::Normal;
                    } else if c == 0x7F as char || c == 0x08 as char {
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

        Ok(())
    }

    fn handle_mouse_input(&mut self) -> Result<(), Error> {
        if let Some(MouseEvent { button, position }) = Api::get_mouse_cursor_info() {
            if button.l() || button.c() || button.r() {
                let relative_pos = (
                    position.x - WINDOW_INIT_X_POS,
                    position.y - WINDOW_INIT_Y_POS,
                );

                // Ignore when click outside the window.
                if relative_pos.0 < 0
                    || relative_pos.0 > WINDOW_WIDTH
                    || relative_pos.1 < 0
                    || relative_pos.1 > WINDOW_HEIGHT
                {
                    println!("button clicked OUTSIDE window: {button:?} {position:?}");

                    return Ok(());
                }

                // Click inside the title bar.
                if relative_pos.1 < TITLE_BAR_HEIGHT {
                    println!("button clicked in title bar: {button:?} {position:?}");
                    self.input_mode = InputMode::Normal;
                    return Ok(());
                }

                if relative_pos.1 < TOOLBAR_HEIGHT + TITLE_BAR_HEIGHT
                    && relative_pos.1 >= TITLE_BAR_HEIGHT
                {
                    self.clear_address_bar()?;
                    self.input_url = String::new();
                    self.input_mode = InputMode::Editing;
                    println!("button clicked in toolbar: {button:?} {position:?}");
                    return Ok(());
                }

                let position_in_content_area = (
                    relative_pos.0,
                    relative_pos.1 - TITLE_BAR_HEIGHT - TOOLBAR_HEIGHT,
                );
                let page = self.browser.borrow().current_page();
                page.borrow_mut().clicked(position_in_content_area);

                self.input_mode = InputMode::Normal;
                println!("button clicked: {button:?} {position:?} {position_in_content_area:?}");

                for log in self.browser.borrow().logs() {
                    print!("{}\n", log.to_string());
                }
                self.browser.borrow_mut().clear_logs();
            }
        }

        Ok(())
    }

    fn run_app(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        loop {
            self.handle_key_input(handle_url)?;
            self.handle_mouse_input()?;
        }
    }

    fn start_navigation_from_toolbar(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
        destination: String,
    ) -> Result<(), Error> {
        match handle_url(destination) {
            Ok(response) => {
                self.browser.borrow_mut().clear_logs();

                let page = self.browser.borrow().current_page();
                page.borrow_mut().clear_display_items();
                page.borrow_mut().receive_response(response);
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
    }

    fn update_ui(&mut self) -> Result<(), Error> {
        let display_items = self
            .browser
            .borrow()
            .current_page()
            .borrow()
            .display_items();

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
                    self.position.1 += CHAR_HEIGHT_WITH_PADDING;
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
                            StringSize::Medium => self.position.1 += CHAR_HEIGHT_WITH_PADDING,
                            StringSize::Large => self.position.1 += CHAR_HEIGHT_WITH_PADDING * 2,
                            StringSize::XLarge => self.position.1 += CHAR_HEIGHT_WITH_PADDING * 3,
                        }
                    }
                }
                DisplayItem::Img {
                    src,
                    style: _,
                    layout_point: _,
                } => {
                    print!("DisplayItem::Img src: {}\n", src);

                    self.browser.borrow_mut().push_url_for_subresource(src);

                    let data = include_bytes!("./test.bmp");
                    let bmp = match Bmp::<Rgb888>::from_slice(data) {
                        Ok(bmp) => bmp,
                        Err(e) => {
                            return Err(Error::Other(format!("failed to draw an image: {:?}", e)))
                        }
                    };
                    let bmp_header = match RawBmp::from_slice(data) {
                        Ok(bmp) => bmp.header().clone(),
                        Err(e) => {
                            return Err(Error::Other(format!("failed to draw an image: {:?}", e)))
                        }
                    };

                    //let img: ImageRawBE<Rgb888> = ImageRaw::new(data, 200);
                    //let image = Image::new(&img, Point::zero());
                    let image = Image::new(
                        &bmp,
                        Point::new(self.position.0 as i32, self.position.1 as i32),
                    );
                    //print!("image: {:#?}\n", image);

                    if image.draw(&mut self.window).is_err() {
                        return Err(Error::Other("failed to draw an image".to_string()));
                    }

                    self.position.1 += bmp_header.image_size.height as i64;
                }
            }
        }

        for log in self.browser.borrow().logs() {
            print!("{}\n", log.to_string());
        }
        self.browser.borrow_mut().clear_logs();

        Ok(())
    }

    fn update_address_bar(&mut self) -> Result<(), Error> {
        self.clear_address_bar()?;

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
