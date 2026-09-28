use std::time::{Duration, Instant};

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use ratatui_textarea::TextArea;
use tokio::sync::oneshot;

mod convert;
use convert::ConversionResult;

#[derive(Debug, Clone, Copy)]
enum Mode {
    Normal,
    FileInput,
    FormatSelect,
}

pub struct Converter<'a> {
    selected: usize,
    current_file: Option<String>,
    current_format: Option<String>,
    format_selected: usize,
    formats: Vec<String>,
    file_input: TextArea<'a>,
    mode: Mode,
    status_message: Option<String>,
    status_until: Option<Instant>,
    converting: bool,
    conversion_rx: Option<oneshot::Receiver<ConversionResult>>,
}

impl<'a> Converter<'a> {
    pub fn new() -> Self {
        let mut file_input = TextArea::default();
        file_input.set_block(Block::default().borders(Borders::ALL).title("Enter File"));

        let formats = vec![
            "mp4".to_string(),
            "mkv".to_string(),
            "avi".to_string(),
            "mov".to_string(),
        ];

        let current_format = Some("mp4".to_string());

        Self {
            selected: 0,
            current_file: None,
            current_format,
            format_selected: 0,
            formats,
            file_input,
            mode: Mode::Normal,
            status_message: None,
            status_until: None,
            converting: false,
            conversion_rx: None,
        }
    }

    pub fn is_converting(&self) -> bool {
        self.converting
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.converting {
            if key.code == KeyCode::Esc {
                self.converting = false;
                self.conversion_rx = None;
                self.status_message = Some("Conversion canceled.".to_string());
                self.status_until = Some(Instant::now() + Duration::from_secs(3));
            }
            return;
        }

        match self.mode {
            Mode::Normal => self.handle_normal_key(key.code),
            Mode::FileInput => self.handle_file_input(key),
            Mode::FormatSelect => self.handle_format_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % 3;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected == 0 {
                    self.selected = 2;
                } else {
                    self.selected -= 1;
                }
            }
            KeyCode::Enter => match self.selected {
                0 => self.start_file_input(),
                1 => self.start_format_select(),
                2 => self.run(),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_file_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                let file = self
                    .file_input
                    .lines()
                    .first()
                    .map(|line| line.trim().to_string())
                    .unwrap_or_default();

                self.mode = Mode::Normal;

                if file.is_empty() {
                    self.current_file = None;
                    return;
                }

                self.current_file = Some(file);
            }
            _ => {
                self.file_input.input(key);
            }
        }
    }

    fn handle_format_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Up => {
                if self.format_selected > 0 {
                    self.format_selected -= 1;
                }
            }
            KeyCode::Down => {
                if !self.formats.is_empty() && self.format_selected + 1 < self.formats.len() {
                    self.format_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(format) = self.formats.get(self.format_selected) {
                    self.current_format = Some(format.clone());
                }
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.poll_status();

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .split(area);

        match self.mode {
            Mode::Normal => {
                let block = Block::bordered();
                let inner = block.inner(sections[0]);
                frame.render_widget(block, sections[0]);

                let items = [
                    format!("Input File: {}", self.current_file.as_deref().unwrap_or("none")),
                    format!("Format: {}", self.current_format.as_deref().unwrap_or("none")),
                    "Convert".to_string(),
                ];

                let list_items = items
                    .iter()
                    .map(|item| ListItem::new(item.as_str()))
                    .collect::<Vec<_>>();

                let list = List::new(list_items)
                    .highlight_symbol("> ")
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD));

                let mut state = ListState::default();
                state.select(Some(self.selected));

                frame.render_stateful_widget(list, inner, &mut state);
            }
            Mode::FileInput => {
                frame.render_widget(&self.file_input, sections[0]);
            }
            Mode::FormatSelect => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title("Select Output Format")
                    .title_bottom("Enter = Select | Esc = Cancel");

                let items: Vec<ListItem> = self
                    .formats
                    .iter()
                    .map(|format| ListItem::new(format.clone()))
                    .collect();

                let list = List::new(items)
                    .block(block)
                    .highlight_symbol(">> ")
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD));

                let mut state = ListState::default();
                if !self.formats.is_empty() {
                    state.select(Some(self.format_selected.min(self.formats.len() - 1)));
                }

                frame.render_stateful_widget(list, sections[0], &mut state);
            }
        }

        self.draw_popup(frame, sections[1]);
    }

    fn draw_popup(&self, frame: &mut Frame, area: Rect) {
        let Some(message) = self.status_message.as_deref() else {
            return;
        };

        let should_show = self.converting
            || self
                .status_until
                .map(|until| Instant::now() < until)
                .unwrap_or(false);

        if !should_show {
            return;
        }

        let title = if self.converting { "Converting..." } else { "Conversion" };
        let popup = Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(popup, area);
    }

    fn poll_status(&mut self) {
        if let Some(until) = self.status_until {
            if Instant::now() >= until {
                self.status_message = None;
                self.status_until = None;
            }
        }

        if self.converting {
            if let Some(rx) = &mut self.conversion_rx {
                match rx.try_recv() {
                    Ok(ConversionResult::Success(msg)) => {
                        self.finish_conversion(msg);
                    }
                    Ok(ConversionResult::Failure(msg)) => {
                        self.finish_conversion(format!("Failed: {}", msg));
                    }
                    Err(oneshot::error::TryRecvError::Empty) => {}
                    Err(oneshot::error::TryRecvError::Closed) => {
                        self.finish_conversion("Conversion task dropped unexpectedly.".to_string());
                    }
                }
            }
        }
    }

    fn finish_conversion(&mut self, message: String) {
        self.conversion_rx = None;
        self.converting = false;
        self.status_message = Some(message);
        self.status_until = Some(Instant::now() + Duration::from_secs(3));
    }

    fn set_status(&mut self, message: String, secs: u64) {
        self.status_message = Some(message);
        self.status_until = Some(Instant::now() + Duration::from_secs(secs));
    }

    fn run(&mut self) {
        if self.converting {
            return;
        }

        let Some(file) = self.current_file.clone() else {
            self.set_status("No input file selected.".to_string(), 2);
            return;
        };

        let Some(format) = self.current_format.clone() else {
            self.set_status("No output format selected.".to_string(), 2);
            return;
        };

        let output_path = convert::build_output_path(&file, &format);
        let (tx, rx) = oneshot::channel::<ConversionResult>();

        tokio::spawn(async move {
            convert::convert_video(file, output_path, tx).await;
        });

        self.status_message = Some("Converting...".to_string());
        self.status_until = None;
        self.converting = true;
        self.conversion_rx = Some(rx);
    }

    fn start_file_input(&mut self) {
        let current = self.current_file.clone().unwrap_or_default();
        self.file_input = TextArea::new(vec![current]);
        self.file_input
            .set_block(Block::default().borders(Borders::ALL).title("Enter File"));
        self.mode = Mode::FileInput;
    }

    fn start_format_select(&mut self) {
        self.format_selected = self
            .formats
            .iter()
            .position(|format| Some(format.as_str()) == self.current_format.as_deref())
            .unwrap_or(0);
        self.mode = Mode::FormatSelect;
    }
}