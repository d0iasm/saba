use crate::cursor::Cursor;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use core::cell::RefCell;
use core::include_bytes;
use embedded_graphics::{image::Image, pixelcolor::Rgb888, prelude::*};
use noli::error::Result as OsResult;
use noli::prelude::SystemApi;
use noli::print;
use noli::println;
use noli::rect::Rect;
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

#[derive(Debug)]
pub struct WasabiUI {
    browser: Rc<RefCell<Browser>>,
    input_url: String,
    input_mode: InputMode,
    window: Window,
    cursor: Cursor,
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
            .expect("failed to create a window"),
            cursor: Cursor::new(),
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

    fn setup(&mut self) -> Result<(), Error> {
        if let Err(error) = self.setup_toolbar() {
            return Err(Error::InvalidUI(format!(
                "failed to initialize a toolbar with error: {:#?}",
                error
            )));
        }
        self.window.flush();
        Ok(())
    }

    fn setup_toolbar(&mut self) -> OsResult<()> {
        self.window
            .fill_rect(LIGHTGREY, 0, 0, WINDOW_WIDTH, TOOLBAR_HEIGHT)?;

        self.window
            .draw_line(GREY, 0, TOOLBAR_HEIGHT, WINDOW_WIDTH - 1, TOOLBAR_HEIGHT)?;
        self.window.draw_line(
            DARKGREY,
            0,
            TOOLBAR_HEIGHT + 1,
            WINDOW_WIDTH - 1,
            TOOLBAR_HEIGHT + 1,
        )?;

        self.window.draw_string(
            BLACK,
            5,
            5,
            "Address:",
            StringSize::Medium,
            /*underline=*/ false,
        )?;

        // address bar
        self.window
            .fill_rect(WHITE, 70, 2, WINDOW_WIDTH - 74, 2 + ADDRESSBAR_HEIGHT)?;

        // shadow for address bar
        self.window.draw_line(GREY, 70, 2, WINDOW_WIDTH - 4, 2)?;
        self.window
            .draw_line(GREY, 70, 2, 70, 2 + ADDRESSBAR_HEIGHT)?;
        self.window.draw_line(BLACK, 71, 3, WINDOW_WIDTH - 5, 3)?;

        self.window
            .draw_line(GREY, 71, 3, 71, 1 + ADDRESSBAR_HEIGHT)?;

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
                    if c == 0x0A as char {
                        // enter key
                        let _ = self.start_navigation(handle_url, "http://example.com".to_string());

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

    fn handle_mouse_input(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        if let Some(MouseEvent { button, position }) = Api::get_mouse_cursor_info() {
            self.window.flush_area(self.cursor.rect());
            self.cursor.set_position(position.x, position.y);
            self.window.flush_area(self.cursor.rect());
            self.cursor.flush();

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

                self.input_mode = InputMode::Normal;

                let position_in_content_area = (
                    relative_pos.0,
                    relative_pos.1 - TITLE_BAR_HEIGHT - TOOLBAR_HEIGHT,
                );
                let page = self.browser.borrow().current_page();
                let next_destination = page.borrow_mut().clicked(position_in_content_area);

                // clear logs.
                for log in self.browser.borrow().logs() {
                    print!("{}\n", log.to_string());
                }
                self.browser.borrow_mut().clear_logs();

                if let Some(url) = next_destination {
                    // navigate to the next url.
                    let _ = self.start_navigation(handle_url, url);
                }
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
            self.handle_mouse_input(handle_url)?;
        }
    }

    fn start_navigation(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
        destination: String,
    ) -> Result<(), Error> {
        self.clear_content_area()?;

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

        self.update_ui()?;

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
                    style,
                    layout_point,
                    layout_size,
                } => {
                    if self
                        .window
                        .fill_rect(
                            style.background_color().code_u32(),
                            layout_point.x() + WINDOW_PADDING,
                            layout_point.y() + WINDOW_PADDING + TOOLBAR_HEIGHT,
                            layout_size.width(),
                            layout_size.height(),
                        )
                        .is_err()
                    {
                        return Err(Error::InvalidUI("failed to draw a string".to_string()));
                    }
                }
                DisplayItem::Text {
                    text,
                    style,
                    layout_point,
                } => {
                    if self
                        .window
                        .draw_string(
                            style.color().code_u32(),
                            layout_point.x() + WINDOW_PADDING,
                            layout_point.y() + WINDOW_PADDING + TOOLBAR_HEIGHT,
                            &text,
                            convert_font_size(style.font_size()),
                            style.text_decoration() == TextDecoration::Underline,
                        )
                        .is_err()
                    {
                        return Err(Error::InvalidUI("failed to draw a string".to_string()));
                    }
                }
                DisplayItem::Img {
                    src,
                    style: _,
                    layout_point,
                } => {
                    print!("DisplayItem::Img src: {}\n", src);

                    self.browser.borrow_mut().push_url_for_subresource(src);

                    let data = include_bytes!("./youtube.bmp");
                    let bmp = match Bmp::<Rgb888>::from_slice(data) {
                        Ok(bmp) => bmp,
                        Err(e) => {
                            return Err(Error::Other(format!("failed to draw an image: {:?}", e)))
                        }
                    };
                    let _bmp_header = match RawBmp::from_slice(data) {
                        Ok(bmp) => bmp.header().clone(),
                        Err(e) => {
                            return Err(Error::Other(format!("failed to draw an image: {:?}", e)))
                        }
                    };

                    let image = Image::new(
                        &bmp,
                        Point::new(
                            (layout_point.x() + WINDOW_PADDING) as i32,
                            (layout_point.y() + WINDOW_PADDING + TOOLBAR_HEIGHT) as i32,
                        ),
                    );
                    //print!("image: {:#?}\n", image);

                    if image.draw(&mut self.window).is_err() {
                        return Err(Error::Other("failed to draw an image".to_string()));
                    }
                }
            }
        }

        for log in self.browser.borrow().logs() {
            print!("{}\n", log.to_string());
        }
        self.browser.borrow_mut().clear_logs();

        self.window.flush();

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
                "failed to update an address bar".to_string(),
            ));
        }

        self.window.flush_area(
            // This rect should be the absolute potision.
            Rect::new(
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS + TITLE_BAR_HEIGHT,
                WINDOW_WIDTH,
                TOOLBAR_HEIGHT,
            )
            .expect("failed to create a rect for the address bar"),
        );

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
                "failed to clear an address bar".to_string(),
            ));
        }

        Ok(())
    }

    fn clear_content_area(&mut self) -> Result<(), Error> {
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
                "failed to clear a content area".to_string(),
            ));
        }

        self.window.flush();

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
