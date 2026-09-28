use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use ratatui_textarea::TextArea;

pub mod download;

enum Mode {
    Normal,
    UrlInput,
}

pub struct Downloader<'a> {
    selected: usize,
    mode: Mode,
    url_input: TextArea<'a>,
    current_url: Option<String>,
}

impl<'a> Downloader<'a> {
    pub fn new() -> Self {
        let mut url_input = TextArea::default();
        url_input.set_block(Block::default().borders(Borders::ALL).title("Enter URL"));

        Self {
            selected: 0,
            mode: Mode::Normal,
            url_input,
            current_url: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.handle_normal_key(key.code),
            Mode::UrlInput => self.handle_url_input(key),
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
                    self.start_url_input();
                }
                1 => {
                    self.run();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_url_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                let url = self
                    .url_input
                    .lines()
                    .first()
                    .map(|line| line.trim().to_string())
                    .unwrap_or_default();

                self.mode = Mode::Normal;

                if url.is_empty() {
                    self.current_url = None;
                    return;
                }

                self.current_url = Some(url);
            }
            _ => {
                self.url_input.input(key);
            }
        }
    }

    fn start_url_input(&mut self) {
        let current = self.current_url.clone().unwrap_or_default();
        self.url_input = TextArea::new(vec![current]);
        self.mode = Mode::UrlInput;
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            Mode::Normal => {
                let block = Block::bordered();
                let inner = block.inner(area);
                frame.render_widget(block, area);

                let items = [
                    format!("URL: {}", self.current_url.as_deref().unwrap_or("none")),
                    "Download".to_string(),
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
            Mode::UrlInput => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title("Enter URL")
                    .title_bottom("Enter = Save | Esc = Cancel");
                let inner = block.inner(area);
                frame.render_widget(block, area);
                frame.render_widget(&self.url_input, inner);
            }
        }
    }

    fn run(&self) {
        let Some(url) = self.current_url.as_deref() else {
            return;
        };

        let file_name = url
            .split('/')
            .next_back()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("downloaded_media");

        let output_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(file_name);

        let _ = download::download_url(url,  output_path.to_str().unwrap_or(file_name));
    }
}
