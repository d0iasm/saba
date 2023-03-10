use alloc::rc::Rc;
use alloc::string::ToString;
use common::error::Error;
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
use renderer::page::page::Page;
use renderer::ui::UiObject;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
enum InputMode {
    Normal,
    Editing,
}

enum LogLevel {
    Debug,
    Error,
}

struct Log {
    level: LogLevel,
    log: String,
}

impl Log {
    fn new(level: LogLevel, log: String) -> Self {
        Self { level, log }
    }
}

impl ToString for Log {
    fn to_string(&self) -> String {
        match self.level {
            LogLevel::Debug => format!("[DEBUG] {}", self.log),
            LogLevel::Error => format!("[ERROR] {}", self.log),
        }
    }
}

pub struct Tui {
    page: Option<Rc<RefCell<Page<Self>>>>,
    input_url: String,
    input_mode: InputMode,
    contents: Vec<String>,
    logs: Vec<Log>,
}

impl UiObject for Tui {
    fn new() -> Self {
        Self {
            page: None,
            input_url: String::new(),
            input_mode: InputMode::Normal,
            contents: Vec::new(),
            logs: Vec::new(),
        }
    }

    fn println(&mut self, text: String) {
        self.contents.push(text);
    }

    fn console_debug(&mut self, log: String) {
        self.logs.push(Log::new(LogLevel::Debug, log));
    }

    fn console_error(&mut self, log: String) {
        self.logs.push(Log::new(LogLevel::Error, log));
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
    pub fn set_page(&mut self, page: Rc<RefCell<Page<Tui>>>) {
        self.page = Some(page);
    }

    pub fn page(&self) -> Option<Rc<RefCell<Page<Self>>>> {
        self.page.clone()
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
                            let url: String = self.input_url.drain(..).collect();
                            self.contents.push(url.clone());
                            match handle_url(url.clone()) {
                                Ok(response) => {
                                    let page = match self.page() {
                                        Some(page) => page,
                                        None => {
                                            return Err(Error::Other(
                                                "associated page is not found".to_string(),
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

        // box for main content
        let contents: Vec<ListItem> = self
            .contents
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                ListItem::new(content)
            })
            .collect();
        let contents =
            List::new(contents).block(Block::default().borders(Borders::ALL).title("Content"));
        frame.render_widget(contents, chunks[2]);

        // box for console logs
        let logs: Vec<ListItem> = self
            .logs
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
