use std::time::{Duration, Instant};

use ratatui::{
    Frame, crossterm::event::{KeyCode, KeyEvent}, layout::{Constraint, Direction, Layout, Rect}, 
    style::{Modifier, Style}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use ratatui_textarea::TextArea;

use crate::subtitles::subtitles::{extract_subtitles, get_subtitle_tracks, Mode, SubtitleFormat, SubtitleTrack};

mod subtitles;


pub struct Subtitles<'a> {
    // Main menu:
    // 0 = File
    // 1 = Format
    // 2 = Tracks
    // 3 = Run
    pub selected: usize,

    // Track selection menu
    pub track_selected: usize,

    // Format selection menu
    pub format_selected: usize,

    pub tracks: Vec<SubtitleTrack>,

    // true = track will be extracted
    pub enabled_tracks: Vec<bool>,

    pub current_file: Option<String>,

    pub format: String,

    pub formats: Vec<String>,

    pub file_input: TextArea<'a>,

    pub mode: Mode,

    // Temporary error/status popup
    pub error_message: Option<String>,
    pub error_until: Option<Instant>,
}

impl<'a> Subtitles<'a> {
    pub fn new() -> Self {
        let mut file_input = TextArea::default();

        file_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter File"),
        );

        Self {
            selected: 0,

            track_selected: 0,

            format_selected: 0,

            tracks: Vec::new(),

            enabled_tracks: Vec::new(),

            // No file pre-selected.
            current_file: None,

            format: "SRT".to_string(),

            formats: vec![
                "SRT".to_string(),
                "ASS".to_string(),
                "VTT".to_string(),
            ],

            file_input,

            mode: Mode::Normal,

            error_message: None,
            error_until: None,
        }
    }
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .split(area);

        match self.mode {
            Mode::Normal => draw_normal(self, frame, sections[0]),
            Mode::FileInput => draw_file_input(self, frame, sections[0]),
            Mode::FormatSelect => draw_format_select(self, frame, sections[0]),
            Mode::TrackSelect => draw_track_select(self, frame, sections[0]),
        }
        draw_popup(self, frame, sections[1]);
    }

    // --------------------------------------------------
    // TICK
    // --------------------------------------------------

    pub fn _tick(&mut self) {
        if let Some(until) = self.error_until {
            if Instant::now() >= until {
                self.error_message = None;
                self.error_until = None;
            }
        }
    }

    // --------------------------------------------------
    // KEY HANDLING
    // --------------------------------------------------

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => {
                self.handle_normal_key(key);
            }

            Mode::FileInput => {
                self.handle_file_input(key);
            }

            Mode::FormatSelect => {
                self.handle_format_key(key);
            }

            Mode::TrackSelect => {
                self.handle_track_key(key);
            }
        }
    }

    // --------------------------------------------------
    // MAIN MENU
    // --------------------------------------------------

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }

            KeyCode::Down => {
                if self.selected < 3 {
                    self.selected += 1;
                }
            }

            KeyCode::Enter => {
                match self.selected {
                    0 => {
                        self.start_file_input();
                    }

                    1 => {
                        self.start_format_select();
                    }

                    2 => {
                        self.start_track_select();
                    }

                    3 => {
                        self.run();
                    }

                    _ => {}
                }
            }

            _ => {}
        }
    }

    // --------------------------------------------------
    // FILE INPUT
    // --------------------------------------------------

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
                    self.tracks.clear();
                    self.enabled_tracks.clear();
                    self.track_selected = 0;

                    return;
                }

                self.current_file = Some(file);

                self.refresh_tracks();
            }

            _ => {
                self.file_input.input(key);
            }
        }
    }

    fn start_file_input(&mut self) {
        let current = self
            .current_file
            .clone()
            .unwrap_or_default();

        self.file_input = TextArea::new(vec![current]);

        self.mode = Mode::FileInput;
    }

    // --------------------------------------------------
    // FORMAT
    // --------------------------------------------------

    fn start_format_select(&mut self) {
        self.format_selected = self
            .formats
            .iter()
            .position(|format| format == &self.format)
            .unwrap_or(0);

        self.mode = Mode::FormatSelect;
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
                if !self.formats.is_empty()
                    && self.format_selected + 1 < self.formats.len()
                {
                    self.format_selected += 1;
                }
            }

            KeyCode::Enter => {
                if let Some(format) =
                    self.formats.get(self.format_selected)
                {
                    self.format = format.clone();
                }

                self.mode = Mode::Normal;
            }

            _ => {}
        }
    }

    // --------------------------------------------------
    // TRACKS
    // --------------------------------------------------

    fn start_track_select(&mut self) {
        if self.tracks.is_empty() {
            self.refresh_tracks();
        }

        if self.tracks.is_empty() {
            self.show_error("No subtitle tracks found.");
            return;
        }

        self.track_selected = self
            .track_selected
            .min(self.tracks.len() - 1);

        self.mode = Mode::TrackSelect;
    }

    fn handle_track_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }

            KeyCode::Up => {
                if self.track_selected > 0 {
                    self.track_selected -= 1;
                }
            }

            KeyCode::Down => {
                if !self.tracks.is_empty()
                    && self.track_selected + 1 < self.tracks.len()
                {
                    self.track_selected += 1;
                }
            }

            KeyCode::Char(' ') => {
                self.toggle_current_track();
            }

            KeyCode::Char('a')
            | KeyCode::Char('A') => {
                self.toggle_all_tracks();
            }

            KeyCode::Enter => {
                self.mode = Mode::Normal;
            }

            _ => {}
        }
    }

    fn toggle_current_track(&mut self) {
        if self.track_selected >= self.enabled_tracks.len() {
            return;
        }

        self.enabled_tracks[self.track_selected] =
            !self.enabled_tracks[self.track_selected];
    }

    fn toggle_all_tracks(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let all_enabled = self.all_tracks_enabled();

        for enabled in &mut self.enabled_tracks {
            *enabled = !all_enabled;
        }
    }

    pub fn all_tracks_enabled(&self) -> bool {
        !self.tracks.is_empty()
            && self.enabled_tracks.len() == self.tracks.len()
            && self.enabled_tracks.iter().all(|enabled| *enabled)
    }

    pub fn has_valid_track_selection(&self) -> bool {
        !self.tracks.is_empty()
            && self
                .enabled_tracks
                .iter()
                .any(|enabled| *enabled)
    }

    // --------------------------------------------------
    // REFRESH TRACKS
    // --------------------------------------------------

    fn refresh_tracks(&mut self) {
        let Some(file) = self.current_file.clone() else {
            self.tracks.clear();
            self.enabled_tracks.clear();
            self.track_selected = 0;

            return;
        };

        match get_subtitle_tracks(&file) {
            Ok(tracks) => {
                self.tracks = tracks;

                self.enabled_tracks =
                    vec![false; self.tracks.len()];

                self.track_selected = 0;
            }

            Err(error) => {
                self.tracks.clear();
                self.enabled_tracks.clear();
                self.track_selected = 0;

                self.show_error(
                    format!(
                        "Failed to read subtitles:\n{}",
                        error
                    ),
                );
            }
        }
    }

    // --------------------------------------------------
    // RUN
    // --------------------------------------------------

    fn run(&mut self) {
        let Some(file) = self.current_file.clone() else {
            self.show_error("No input file selected.");
            return;
        };

        if self.tracks.is_empty() {
            self.show_error("No subtitle tracks found.");
            return;
        }

        if !self.has_valid_track_selection() {
            self.show_error("No subtitle tracks selected.");
            return;
        }

        let format = match self.format.as_str() {
            "SRT" => SubtitleFormat::Srt,
            "ASS" => SubtitleFormat::Ass,
            "VTT" => SubtitleFormat::Vtt,

            _ => {
                self.show_error("Unsupported subtitle format.");
                return;
            }
        };

        let input_path = std::path::Path::new(&file);

        let output_directory = input_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));

        let file_stem = input_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("subtitle");

        let output_file = output_directory.join(format!(
            "{}.{}",
            file_stem,
            format.extension()
        ));

        let output_path = output_file.to_string_lossy();

        match extract_subtitles(
            &file,
            format,
            &output_path,
        ) {
            Ok(()) => {
                self.show_error(
                    format!(
                        "Subtitle extracted:\n{}",
                        output_path
                    ),
                );
            }

            Err(error) => {
                self.show_error(
                    format!(
                        "Subtitle extraction failed:\n{}",
                        error
                    ),
                );
            }
        }
    }

    // --------------------------------------------------
    // SELECTED TRACKS
    // --------------------------------------------------

    pub fn _selected_tracks(&self) -> Vec<&SubtitleTrack> {
        self.tracks
            .iter()
            .enumerate()
            .filter_map(|(index, track)| {
                self.enabled_tracks
                    .get(index)
                    .copied()
                    .unwrap_or(false)
                    .then_some(track)
            })
            .collect()
    }

    // --------------------------------------------------
    // FILE EDITING
    // --------------------------------------------------

    pub fn file_editing(&self) -> bool {
        matches!(self.mode, Mode::FileInput)
    }

    // --------------------------------------------------
    // ERROR / STATUS
    // --------------------------------------------------

    fn show_error(
        &mut self,
        message: impl Into<String>,
    ) {
        self.error_message = Some(message.into());

        self.error_until = Some(
            Instant::now() + Duration::from_secs(2),
        );
    }
}

fn draw_normal(subtitles: &Subtitles, frame: &mut Frame, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(area);

    draw_file(subtitles, frame, rows[0]);
    draw_format(subtitles, frame, rows[1]);
    draw_tracks(subtitles, frame, rows[2]);
    draw_run(subtitles, frame, rows[3]);
}

fn draw_file(subtitles: &Subtitles, frame: &mut Frame, area: Rect) {
    let value = subtitles
        .current_file
        .as_deref()
        .unwrap_or("No file selected");

    let prefix = if subtitles.selected == 0 {
        ">> "
    } else {
        "   "
    };

    let style = if subtitles.selected == 0 {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    frame.render_widget(
        Paragraph::new(format!("{}File:   [ {} ]", prefix, value)).style(style),
        area,
    );
}

fn draw_format(subtitles: &Subtitles, frame: &mut Frame, area: Rect) {
    let prefix = if subtitles.selected == 1 {
        ">> "
    } else {
        "   "
    };

    let style = if subtitles.selected == 1 {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    frame.render_widget(
        Paragraph::new(format!("{}Format: [ {} ]", prefix, subtitles.format)).style(style),
        area,
    );
}

fn draw_tracks(subtitles: &Subtitles, frame: &mut Frame, area: Rect) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(area);

    let prefix = if subtitles.selected == 2 {
        ">> "
    } else {
        "   "
    };

    let track_status = if subtitles.has_valid_track_selection() {
        "[x]"
    } else {
        "[ ]"
    };

    let header = format!("{}Tracks: {}", prefix, track_status);

    let style = if subtitles.selected == 2 {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    frame.render_widget(Paragraph::new(header).style(style), sections[0]);

    if subtitles.tracks.is_empty() {
        frame.render_widget(Paragraph::new("   No subtitle tracks found"), sections[1]);
        return;
    }

    let header = ListItem::new("   ID     Language       Title")
        .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = subtitles.tracks.iter().enumerate().map(|(index, track)| {
        let enabled = subtitles
            .enabled_tracks
            .get(index)
            .copied()
            .unwrap_or(false);

        let checkbox = if enabled { "[x]" } else { "[ ]" };

        let language = if track.language.is_empty() {
            "Unknown"
        } else {
            &track.language
        };

        let title = if track.title.is_empty() {
            "Untitled"
        } else {
            &track.title
        };

        ListItem::new(format!(
            "   {} {:<6} {:<14} {}",
            checkbox, track.index, language, title,
        ))
    });

    let items: Vec<ListItem> = std::iter::once(header).chain(rows).collect();

    let list = List::new(items);

    frame.render_widget(list, sections[1]);
}

fn draw_run(subtitles: &Subtitles, frame: &mut Frame, area: Rect) {
    let prefix = if subtitles.selected == 3 {
        ">> "
    } else {
        "   "
    };

    let style = if subtitles.selected == 3 {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    frame.render_widget(
        Paragraph::new(format!("{}[ Run ]", prefix)).style(style),
        area,
    );
}

fn draw_popup(subtitles: &mut Subtitles, frame: &mut Frame, area: Rect) {
    let Some(message) = subtitles.error_message.as_deref() else {
        return;
    };

    let Some(until) = subtitles.error_until else {
        return;
    };

    if Instant::now() >= until {
        subtitles.error_message = None;
        subtitles.error_until = None;
        return;
    }

    let popup =
        Paragraph::new(message).block(Block::default().borders(Borders::ALL));

    frame.render_widget(popup, area);
}

fn draw_file_input(subtitles: &mut Subtitles, frame: &mut Frame, area: Rect) {
    subtitles.file_input.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Enter Video File")
            .title_bottom("Enter = Save | Esc = Cancel"),
    );

    frame.render_widget(&subtitles.file_input, area);
}

fn draw_format_select(subtitles: &Subtitles, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Select Subtitle Format")
        .title_bottom("Enter = Select | Esc = Cancel");

    let items: Vec<ListItem> = subtitles
        .formats
        .iter()
        .map(|format| ListItem::new(format.clone()))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_symbol(">> ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = ListState::default();

    if !subtitles.formats.is_empty() {
        state.select(Some(
            subtitles.format_selected.min(subtitles.formats.len() - 1),
        ));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_track_select(subtitles: &Subtitles, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Select Subtitle Tracks")
        .title_bottom(
            "Space = Toggle | A = Toggle All | \
             Enter = Done | Esc = Cancel",
        );

    let mut items = Vec::new();

    let all_enabled = subtitles.all_tracks_enabled();

    let toggle_all_text = if all_enabled {
        "[x] Toggle All"
    } else {
        "[ ] Toggle All"
    };

    items.push(ListItem::new(toggle_all_text).style(Style::default().add_modifier(Modifier::BOLD)));

    for (index, track) in subtitles.tracks.iter().enumerate() {
        let enabled = subtitles
            .enabled_tracks
            .get(index)
            .copied()
            .unwrap_or(false);

        let checkbox = if enabled { "[x]" } else { "[ ]" };

        let language = if track.language.is_empty() {
            "Unknown"
        } else {
            &track.language
        };

        let title = if track.title.is_empty() {
            "Untitled"
        } else {
            &track.title
        };

        items.push(ListItem::new(format!(
            "{} {} - {} ({})",
            checkbox, track.index, title, language
        )));
    }

    let list = List::new(items)
        .block(block)
        .highlight_symbol(">> ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = ListState::default();

    if !subtitles.tracks.is_empty() {
        state.select(Some(subtitles.track_selected + 1));
    } else {
        state.select(Some(0));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

