use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use ratatui_textarea::TextArea;

pub mod extract;

enum Mode {
    Normal,
    FileInput,
}

pub struct Extractor<'a> {
    selected: usize,
    mode: Mode,
    file_input: TextArea<'a>,
    current_file: Option<String>,
}

impl<'a> Extractor<'a> {
    pub fn new() -> Self {
        let mut file_input = TextArea::default();

        file_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter File"),
        );

        Self {
            selected: 0,
            mode: Mode::Normal,
            file_input,
            current_file: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
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
                0 => {
                    self.start_file_input();
                }

                1 => {
                    self.run();
                }

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
        self.mode = Mode::FileInput;
    }

    fn run(&self) {
        let Some(file) = self.current_file.as_deref() else {
            return;
        };

        let _ = extract::extract_frames(file, "frame_%04d.png");
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            Mode::Normal => {
                let block = Block::bordered();
                let inner = block.inner(area);

                frame.render_widget(block, area);

                let items = [
                    format!(
                        "File: {}",
                        self.current_file.as_deref().unwrap_or("none")
                    ),
                    "Extract".to_string(),
                ];

                let list_items = items
                    .iter()
                    .map(|item| ListItem::new(item.as_str()))
                    .collect::<Vec<_>>();

                let list = List::new(list_items)
                    .highlight_symbol("> ")
                    .highlight_style(
                        Style::default().add_modifier(Modifier::BOLD)
                    );

                let mut state = ListState::default();
                state.select(Some(self.selected));

                frame.render_stateful_widget(list, inner, &mut state);
            }

            Mode::FileInput => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title("Enter File")
                    .title_bottom("Enter = Save | Esc = Cancel");

                let inner = block.inner(area);

                frame.render_widget(block, area);
                frame.render_widget(&self.file_input, inner);
            }
        }
    }
}