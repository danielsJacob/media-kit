use ratatui::{
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

pub mod remux;

pub struct Remux {
    selected: usize,
}

impl Remux {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % 3;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {}
            _ => {}
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered();
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let items = ["File", "Container", "Remux"];
        let list = List::new(items.map(ListItem::new))
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut state = ListState::default();
        state.select(Some(self.selected));
        frame.render_stateful_widget(list, inner, &mut state);
    }
}
