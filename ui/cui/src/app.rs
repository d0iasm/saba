use alloc::rc::Weak;
use alloc::string::ToString;
use core::cell::RefCell;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use saba_core::browser::Browser;
use saba_core::http::HttpResponse;
use saba_core::renderer::layout::computed_style::FontSize;
use saba_core::utils::*;
use saba_core::{display_item::DisplayItem, error::Error};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug)]
enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone, Debug)]
struct Link {
    text: String,
    destination: String,
}

impl Link {
    fn new(text: String, destination: String) -> Self {
        Self { text, destination }
    }
}

#[derive(Clone, Debug)]
pub struct Tui {
    browser: Weak<RefCell<Browser>>,
    input_url: String,
    input_mode: InputMode,
    // A user can focus only a link now.
    focus: Option<Link>,
    // The position that starts rendering a next display item.
    //position: (f64, f64),
}

impl Tui {
    pub fn new() -> Self {
        Self {
            browser: Weak::new(),
            input_url: String::new(),
            input_mode: InputMode::Normal,
            focus: None,
            //position: (0.0, 0.0),
        }
    }

    pub fn start(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        // set up terminal
        match enable_raw_mode() {
            Ok(_) => {}
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }

        let mut stdout = io::stdout();
        match execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            Ok(_) => {}
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }
        match execute!(stdout, Clear(ClearType::All)) {
            Ok(_) => {}
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        };
        match size() {
            Ok((cols, rows)) => {
                console_debug(
                    self.browser.clone(),
                    format!("cols rows {:?} {:?}", cols, rows),
                );
            }
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        };

        // never return unless a user quit the tui app
        let result = self.run_app(handle_url, &mut terminal);

        // restore terminal
        match disable_raw_mode() {
            Ok(_) => {}
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }
        match execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ) {
            Ok(_) => {}
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }
        match terminal.show_cursor() {
            Ok(_) => {}
            Err(e) => return Err(Error::Other(format!("{:?}", e))),
        }

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Other(format!("{:?}", e))),
        }
    }

    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser>>) {
        self.browser = browser;
    }

    pub fn browser(&self) -> Weak<RefCell<Browser>> {
        self.browser.clone()
    }

    fn move_focus_to_up(&mut self) {
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };
        let display_items = browser.borrow().display_items();

        let mut previous_link_item: Option<Link> = None;
        for item in display_items {
            match item {
                DisplayItem::Link {
                    text,
                    destination,
                    style: _,
                    layout_point: _,
                } => match &self.focus {
                    Some(current_focus_item) => {
                        if current_focus_item.text == text
                            && current_focus_item.destination == destination
                        {
                            if let Some(prev_link_item) = previous_link_item {
                                self.focus = Some(prev_link_item);
                                return;
                            } else {
                                self.focus = None;
                                return;
                            }
                        }
                        previous_link_item = Some(current_focus_item.clone());
                    }
                    None => {
                        return;
                    }
                },
                _ => {}
            }
        }
    }

    fn move_focus_to_down(&mut self) {
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };
        let display_items = browser.borrow().display_items();

        let mut focus_item_found = false;
        for item in display_items {
            match item {
                DisplayItem::Link {
                    text,
                    destination,
                    style: _,
                    layout_point: _,
                } => match &self.focus {
                    Some(current_focus_item) => {
                        if focus_item_found {
                            self.focus = Some(Link::new(text, destination));
                            return;
                        }

                        if current_focus_item.text == text
                            && current_focus_item.destination == destination
                        {
                            focus_item_found = true;
                        }
                    }
                    None => {
                        self.focus = Some(Link::new(text, destination));
                        return;
                    }
                },
                _ => {}
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
                console_error(self.browser.clone(), format!("{:?}", e));
                return Err(e);
            }
        }
        Ok(())
    }

    /*
    fn push_key_event(&mut self, key_code: KeyCode) {
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };

        // https://docs.rs/crossterm/latest/crossterm/event/enum.KeyCode.html
        let key = match key_code {
            KeyCode::Char(c) => c.to_string(),
            _ => {
                // TODO: propagate backspace key to browser?
                console_debug(self.browser.clone(), format!("{:?} is pressed", key_code));
                return;
            }
        };
    }
    */

    fn run_app<B: Backend>(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Error> {
        loop {
            match terminal.draw(|frame| self.ui(frame)) {
                Ok(_) => {}
                Err(e) => return Err(Error::Other(format!("{:?}", e))),
            }

            let event = match event::read() {
                Ok(event) => event,
                Err(e) => return Err(Error::Other(format!("{:?}", e))),
            };

            match event {
                Event::Key(key) => {
                    //self.push_key_event(key.code);

                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Up => {
                                self.move_focus_to_up();
                            }
                            KeyCode::Down => {
                                self.move_focus_to_down();
                            }
                            KeyCode::Enter => {
                                // do nothing when there is no focused item;
                                if self.focus.is_none() {
                                    continue;
                                }

                                if let Some(focus_item) = &self.focus {
                                    self.start_navigation(
                                        handle_url,
                                        focus_item.destination.clone(),
                                    )?;
                                }
                            }
                            KeyCode::Char('e') => {
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => {
                                // do nothing when a user puts an enter button but URL is empty
                                if self.input_url.len() == 0 {
                                    continue;
                                }

                                let url: String = self.input_url.drain(..).collect();
                                self.start_navigation(handle_url, url.clone())?;
                            }
                            KeyCode::Char(c) => {
                                self.input_url.push(c);
                            }
                            KeyCode::Backspace => {
                                self.input_url.pop();
                            }
                            KeyCode::Esc => {
                                self.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    }
                }
                Event::Mouse(_) => {
                    // Do not support mouse event in Tui browser.
                }
                _ => {}
            }
        }
    }

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(3),
                    Constraint::Percentage(7),
                    Constraint::Percentage(50),
                    Constraint::Percentage(40),
                ]
                .as_ref(),
            )
            .split(frame.size());

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled(
                        "↑ (up arrow)",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to move up a focused link, "),
                    Span::styled(
                        "↓ (down arrow)",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to move down a focused link, "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to start editing, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to navigation to a focused link."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to stop editing, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to navigation."),
                ],
                Style::default(),
            ),
        };
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, chunks[0]);

        // box for url bar
        {
            let input = Paragraph::new(self.input_url.as_ref())
                .style(match self.input_mode {
                    InputMode::Normal => Style::default().fg(Color::White),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                })
                .block(Block::default().borders(Borders::ALL).title("URL"));
            frame.render_widget(input, chunks[1]);
        }
        match self.input_mode {
            InputMode::Normal =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}

            InputMode::Editing => {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                frame.set_cursor(
                    // Put cursor past the end of the input text
                    chunks[1].x + self.input_url.width() as u16 + 1,
                    // Move one line down, from the border to the input line
                    chunks[1].y + 1,
                )
            }
        }

        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };
        let display_items = browser.borrow().display_items();

        /*
        let content_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); display_items.len() + 1])
            .split(chunks[2]);
        let content = Block::default().title("Content").borders(Borders::ALL);
        frame.render_widget(content, chunks[2]);
        */

        let mut spans: Vec<Spans> = Vec::new();

        //let mut i = 0;
        for item in display_items {
            match item {
                DisplayItem::Rect {
                    style: _,
                    layout_point: _,
                    layout_size: _,
                } => {
                    // Do not support positioning in Browser w/ Tui

                    /*
                    self.position = (layout_point.x(), layout_point.y());
                    let block = Block::default().style(Style::default().bg(Color::Green));
                    frame.render_widget(block, content_area[i]);
                    i = i + 1;
                    */
                }
                DisplayItem::Link {
                    text,
                    destination,
                    style: _,
                    layout_point: _,
                } => {
                    if let Some(focus_item) = &self.focus {
                        if focus_item.text == text && focus_item.destination == destination {
                            spans.push(Spans::from(Span::styled(
                                text,
                                Style::default()
                                    .fg(Color::Blue)
                                    .add_modifier(Modifier::UNDERLINED),
                            )));
                            continue;
                        }
                    }
                    spans.push(Spans::from(Span::styled(
                        text,
                        Style::default().fg(Color::Blue),
                    )));
                }
                DisplayItem::Text {
                    text,
                    style,
                    layout_point: _,
                } => {
                    for line in text.split("\n") {
                        spans.push(if style.font_size() != FontSize::Medium {
                            Spans::from(Span::styled(
                                String::from(line),
                                Style::default().add_modifier(Modifier::BOLD),
                            ))
                        } else {
                            Spans::from(Span::raw(String::from(line)))
                        });
                    }
                }
                DisplayItem::Img {
                    src: _,
                    style: _,
                    layout_point: _,
                } => {
                    // Do not support images in CUI.
                }
            }
        }

        let contents = Paragraph::new(spans)
            .block(Block::default().title("Content").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(contents, chunks[2]);

        let logs: Vec<ListItem> = browser
            .borrow()
            .logs()
            .iter()
            .enumerate()
            .map(|(_, log)| {
                let content = vec![Spans::from(Span::raw(format!("{}", log.to_string())))];
                ListItem::new(content)
            })
            .collect();
        let logs = List::new(logs).block(Block::default().borders(Borders::ALL).title("Console"));
        frame.render_widget(logs, chunks[3]);
    }
}
