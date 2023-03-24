use alloc::rc::Weak;
use alloc::string::ToString;
use core::cell::RefCell;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use net::http::HttpResponse;
use std::io;
use toybr_core::browser::Browser;
use toybr_core::common::{display_item::DisplayItem, error::Error, ui::UiObject};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
enum InputMode {
    Normal,
    Editing,
}

pub struct Tui {
    browser: Weak<RefCell<Browser<Self>>>,
    input_url: String,
    input_mode: InputMode,
}

impl UiObject for Tui {
    fn new() -> Self {
        Self {
            browser: Weak::new(),
            input_url: String::new(),
            input_mode: InputMode::Normal,
        }
    }

    // TODO: remove this?
    fn println(&mut self, _text: String) {
        /*
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };

        browser.borrow_mut().println(text);
        */
    }

    fn console_debug(&mut self, log: String) {
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };

        browser.borrow_mut().console_debug(log);
    }

    fn console_warning(&mut self, log: String) {
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };

        browser.borrow_mut().console_warning(log);
    }

    fn console_error(&mut self, log: String) {
        let browser = match self.browser().upgrade() {
            Some(browser) => browser,
            None => return,
        };

        browser.borrow_mut().console_error(log);
    }

    fn start(
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
}

impl Tui {
    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser<Tui>>>) {
        self.browser = browser;
    }

    pub fn browser(&self) -> Weak<RefCell<Browser<Self>>> {
        self.browser.clone()
    }

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

            let current_input_mode = self.input_mode;

            if let Event::Key(key) = event {
                match current_input_mode {
                    InputMode::Normal => match key.code {
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
                            match handle_url(url.clone()) {
                                Ok(response) => {
                                    self.console_debug(format!("received response {:?}", response));

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
                                            return Err(Error::Other(
                                                "associated browser is not found".to_string(),
                                            ))
                                        }
                                    };

                                    page.borrow_mut().receive_response(response);
                                }
                                Err(e) => {
                                    self.console_error(format!("{:?}", e));
                                    return Err(e);
                                }
                            }
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
        }
    }

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            //.margin(2)
            .constraints(
                [
                    Constraint::Percentage(5),
                    Constraint::Percentage(5),
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
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to start editing."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to stop editing, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to record the message"),
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
                    InputMode::Normal => Style::default(),
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

        // support only text now
        let display_items = browser.borrow().display_items();

        let mut spans: Vec<Spans> = Vec::new();
        for item in display_items {
            match item {
                DisplayItem::Rect {
                    style: _,
                    position: _,
                } => {}
                DisplayItem::Link {
                    text: _,
                    destination: _,
                } => {}
                DisplayItem::Text {
                    text,
                    style: _,
                    position: _,
                } => {
                    self.console_debug(text.clone());
                    // TODO: split with "\n" to insert a new line?
                    spans.push(Spans::from(Span::raw(text)));
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
