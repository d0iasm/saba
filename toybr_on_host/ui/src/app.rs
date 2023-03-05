/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use renderer::layout::render_tree::RenderTree;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

pub struct Browser {
    input_url: String,
    input_mode: InputMode,
    messages: Vec<String>,
    logs: Vec<String>,
}

impl Browser {
    pub fn new() -> Self {
        Self {
            input_url: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            logs: Vec::new(),
        }
    }

    pub fn print(text: &str) {
        print!("{}", text);
    }

    pub fn println(text: &str) {
        println!("{}", text);
    }

    pub fn console_debug(&mut self, log: &str) {
        self.logs.push(log.to_string());
    }

    pub fn console_error(&mut self, log: &str) {
        self.logs.push(log.to_string());
    }

    pub fn start(&mut self, handle_input: fn(String) -> RenderTree) -> Result<(), Box<dyn Error>> {
        // set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        execute!(stdout, Clear(ClearType::All))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // never return unless a user quit the browser app
        let res = self.run_app(&mut terminal, handle_input);

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }

    fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        handle_input: fn(String) -> RenderTree,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.ui(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
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
                            handle_input(url.clone());
                            self.messages.push(url);
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
                    Constraint::Percentage(3),
                    Constraint::Percentage(10),
                    Constraint::Percentage(67),
                    Constraint::Percentage(20),
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
        let input = Paragraph::new(self.input_url.as_ref())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("URL"));
        frame.render_widget(input, chunks[1]);
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
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                ListItem::new(content)
            })
            .collect();
        let messages =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Content"));
        frame.render_widget(messages, chunks[2]);

        // box for console logs
        let input = Paragraph::new(self.input_url.as_ref())
            .block(Block::default().borders(Borders::ALL).title("Console"));
        frame.render_widget(input, chunks[3]);
    }
}
