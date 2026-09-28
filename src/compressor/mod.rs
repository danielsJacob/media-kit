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

pub mod compress;

use crate::compressor::compress::CompressionResult;

#[derive(Debug, Clone, Copy)]
enum Mode {
    Normal,
    FileInput,
}

pub struct Compressor<'a> {
    selected: usize,
    current_file: Option<String>,
    file_input: TextArea<'a>,
    mode: Mode,
    status_message: Option<String>,
    status_until: Option<Instant>,
    compressing: bool,
    compression_rx: Option<oneshot::Receiver<CompressionResult>>, // <-- changed
}

impl<'a> Compressor<'a> {
    pub fn new() -> Self {
        let mut file_input = TextArea::default();
        file_input.set_block(Block::default().borders(Borders::ALL).title("Enter File"));

        Self {
            selected: 0,
            current_file: None,
            file_input,
            mode: Mode::Normal,
            status_message: None,
            status_until: None,
            compressing: false,
            compression_rx: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.compressing {
            if key.code == KeyCode::Esc {
                self.compressing = false;
                self.compression_rx = None;
                self.status_message = Some("Compression canceled.".to_string());
                self.status_until = Some(Instant::now() + Duration::from_secs(3));
            }
            return;
        }

        match self.mode {
            Mode::Normal => self.handle_normal_key(key.code),
            Mode::FileInput => self.handle_file_input(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % 2;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected == 0 {
                    self.selected = 1;
                } else {
                    self.selected -= 1;
                }
            }
            KeyCode::Enter => match self.selected {
                0 => self.start_file_input(),
                1 => self.run(),
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

                let items = ["Input File", "Compress"];
                let list_items = items
                    .iter()
                    .map(|item| ListItem::new(*item))
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
        }

        self.draw_popup(frame, sections[1]);
    }

    fn draw_popup(&self, frame: &mut Frame, area: Rect) {
        let Some(message) = self.status_message.as_deref() else {
            return;
        };

        // Show if actively compressing OR if status timer is still active
        let should_show = self.compressing
            || self
                .status_until
                .map(|until| Instant::now() < until)
                .unwrap_or(false);

        if !should_show {
            return;
        }

        let title = if self.compressing {
            "Compressing..."
        } else {
            "Compression"
        };
        let popup =
            Paragraph::new(message).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(popup, area);
    }

    fn poll_status(&mut self) {
        if let Some(until) = self.status_until {
            if Instant::now() >= until {
                self.status_message = None;
                self.status_until = None;
            }
        }

        if self.compressing {
            if let Some(rx) = &mut self.compression_rx {
                match rx.try_recv() {
                    Ok(CompressionResult::Success(msg)) => {
                        self.finish_compression(msg);
                    }
                    Ok(CompressionResult::Failure(msg)) => {
                        self.finish_compression(format!("Failed: {}", msg));
                    }
                    Err(oneshot::error::TryRecvError::Empty) => {}
                    Err(oneshot::error::TryRecvError::Closed) => {
                        self.finish_compression(
                            "Compression task dropped unexpectedly.".to_string(),
                        );
                    }
                }
            }
        }
    }

    fn finish_compression(&mut self, message: String) {
        self.compression_rx = None;
        self.compressing = false;
        self.status_message = Some(message);
        self.status_until = Some(Instant::now() + Duration::from_secs(3));
    }

    fn set_status(&mut self, message: String, secs: u64) {
        self.status_message = Some(message);
        self.status_until = Some(Instant::now() + Duration::from_secs(secs));
    }

    fn run(&mut self) {
        if self.compressing {
            return;
        }

        let Some(file) = self.current_file.clone() else {
            self.set_status("No input file selected.".to_string(), 2);
            return;
        };

        let output_path = compress::build_output_path(&file);
        let (tx, rx) = oneshot::channel::<CompressionResult>();

        tokio::spawn(async move {
            compress::compress_video(file, output_path, tx).await;
        });

        self.status_message = Some("Compressing...".to_string());
        self.status_until = None;
        self.compressing = true;
        self.compression_rx = Some(rx);
    }

    fn start_file_input(&mut self) {
        let current = self.current_file.clone().unwrap_or_default();
        self.file_input = TextArea::new(vec![current]);
        self.file_input
            .set_block(Block::default().borders(Borders::ALL).title("Enter File"));
        self.mode = Mode::FileInput;
    }
    pub fn is_compressing(&self) -> bool {
        self.compressing
    }
}
