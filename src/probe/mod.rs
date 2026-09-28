use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, List, ListItem, ListState},
    Frame,
};
use ratatui_textarea::TextArea;

pub mod probe;

#[derive(Debug, Clone, Copy)]
enum Mode {
    Normal,
    FileInput,
    Analyze,
    Streams,
    Duration,
}

pub struct Probe<'a> {
    selected: usize,

    current_file: Option<String>,

    file_input: TextArea<'a>,

    mode: Mode,

    analyze_result: Option<String>,
    streams_result: Option<serde_json::Value>,
    duration_result: Option<f64>,
}

impl<'a> Probe<'a> {
    pub fn new() -> Self {
        let mut file_input = TextArea::default();

        file_input.set_block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Enter File"),
        );

        Self {
            selected: 0,
            current_file: None,
            file_input,
            mode: Mode::Normal,
            analyze_result: None,
            streams_result: None,
            duration_result: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.handle_normal_key(key.code),
            Mode::FileInput => self.handle_file_input(key),
            Mode::Analyze => {
                if key.code == KeyCode::Esc {
                    self.mode = Mode::Normal;
                }
            }
            Mode::Streams => {
                if key.code == KeyCode::Esc {
                    self.mode = Mode::Normal;
                }
            }
            Mode::Duration => {
                if key.code == KeyCode::Esc {
                    self.mode = Mode::Normal;
                }
            }
        }
    }

    fn handle_normal_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % 4;
            }

            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected == 0 {
                    self.selected = 3;
                } else {
                    self.selected -= 1;
                }
            }

            KeyCode::Enter => match self.selected {
                0 => self.start_file_input(),
                1 => self.run_analyze(),
                2 => self.run_streams(),
                3 => self.run_duration(),
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

    fn start_file_input(&mut self) {
        let current = self.current_file.clone().unwrap_or_default();

        self.file_input = TextArea::new(vec![current]);

        self.file_input.set_block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Enter File"),
        );

        self.mode = Mode::FileInput;
    }

    fn run_analyze(&mut self) {
        let Some(file) = self.current_file.as_deref() else {
            self.analyze_result = Some("No input file selected.".to_string());
            self.mode = Mode::Analyze;
            return;
        };

        self.analyze_result = Some(match probe::probe_file(file) {
            Ok(result) => result,
            Err(error) => format!("Error: {error}"),
        });

        self.mode = Mode::Analyze;
    }

    fn run_streams(&mut self) {
        let Some(file) = self.current_file.as_deref() else {
            self.streams_result = None;
            self.mode = Mode::Streams;
            return;
        };

        self.streams_result = match probe::get_stream_info(file) {
            Ok(result) => Some(result),
            Err(error) => Some(serde_json::json!({
                "error": error.to_string()
            })),
        };

        self.mode = Mode::Streams;
    }

    fn run_duration(&mut self) {
        let Some(file) = self.current_file.as_deref() else {
            self.duration_result = None;
            self.mode = Mode::Duration;
            return;
        };

        self.duration_result = match probe::get_duration(file) {
            Ok(result) => Some(result),
            Err(_) => None,
        };

        self.mode = Mode::Duration;
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            Mode::Normal => self.draw_normal(frame, area),
            Mode::FileInput => self.draw_file_input(frame, area),
            Mode::Analyze => self.draw_analyze(frame, area),
            Mode::Streams => self.draw_streams(frame, area),
            Mode::Duration => self.draw_duration(frame, area),
        }
    }

    fn draw_normal(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered();
        let inner = block.inner(area);

        frame.render_widget(block, area);

        let items = [
            format!(
                "Input File: {}",
                self.current_file.as_deref().unwrap_or("none")
            ),
            "Analyze".to_string(),
            "Streams".to_string(),
            "Duration".to_string(),
        ];

        let list_items = items
            .iter()
            .map(|item| ListItem::new(item.as_str()))
            .collect::<Vec<_>>();

        let list = List::new(list_items)
            .highlight_symbol("> ")
            .highlight_style(
                Style::default().add_modifier(Modifier::BOLD),
            );

        let mut state = ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(list, inner, &mut state);
    }

    fn draw_file_input(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.file_input, area);
    }

    fn draw_analyze(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title("Analyze");

        let inner = block.inner(area);

        frame.render_widget(block, area);

        let text = self
            .analyze_result
            .as_deref()
            .unwrap_or("No result.");

        let paragraph = ratatui::widgets::Paragraph::new(text);

        frame.render_widget(paragraph, inner);
    }

    fn draw_streams(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title("Streams");

        let inner = block.inner(area);

        frame.render_widget(block, area);

        let text = self
            .streams_result
            .as_ref()
            .map(|value| {
                serde_json::to_string_pretty(value)
                    .unwrap_or_else(|_| "Failed to format JSON.".to_string())
            })
            .unwrap_or_else(|| "No result.".to_string());

        let paragraph = ratatui::widgets::Paragraph::new(text);

        frame.render_widget(paragraph, inner);
    }

    fn draw_duration(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title("Duration");

        let inner = block.inner(area);

        frame.render_widget(block, area);

        let text = match self.duration_result {
            Some(duration) => {
                format!("Duration: {:.2} seconds", duration)
            }

            None => "No duration available.".to_string(),
        };

        let paragraph = ratatui::widgets::Paragraph::new(text);

        frame.render_widget(paragraph, inner);
    }
}