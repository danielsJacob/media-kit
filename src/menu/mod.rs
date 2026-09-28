use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use crate::{
    compressor::Compressor, converter::Converter, downloader::Downloader, extractor::Extractor,
    ffmpeg::FFFamily, probe::Probe, reformatter::Reformatter, remux::Remux, subtitles::Subtitles,
    thumbnail::Thumbnail,
};

const LOGO: &str = r#"                                  ░██ ░██              ░██       ░██   ░██
                                  ░██                  ░██             ░██
░█████████████   ░███████   ░████████ ░██ ░██████      ░██    ░██░██░████████
░██   ░██   ░██ ░██    ░██ ░██    ░██ ░██      ░██     ░██   ░██ ░██   ░██
░██   ░██   ░██ ░█████████ ░██    ░██ ░██ ░███████     ░███████  ░██   ░██
░██   ░██   ░██ ░██        ░██   ░███ ░██░██   ░██     ░██   ░██ ░██   ░██
░██   ░██   ░██  ░███████   ░█████░██ ░██ ░█████░██    ░██    ░██░██    ░████
                                                                              "#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuItem {
    Compressor,
    Converter,
    Downloader,
    Extractor,
    Probe,
    Reformatter,
    Remux,
    Subtitles,
    Thumbnail,
    FFmpeg,
}

impl MenuItem {
    pub fn title(self) -> &'static str {
        match self {
            Self::Compressor => "Compressor",
            Self::Converter => "Converter",
            Self::Downloader => "Downloader",
            Self::Extractor => "Extractor",
            Self::Probe => "Probe",
            Self::Reformatter => "Reformatter",
            Self::Remux => "Remux",
            Self::Subtitles => "Subtitles",
            Self::Thumbnail => "Thumbnail",
            Self::FFmpeg => "FFmpeg",
        }
    }

    pub fn index(self) -> usize {
        Self::all()
            .iter()
            .position(|item| *item == self)
            .unwrap_or(0)
    }

    pub fn all() -> [Self; 10] {
        [
            Self::Compressor,
            Self::Converter,
            Self::Downloader,
            Self::Extractor,
            Self::Probe,
            Self::Reformatter,
            Self::Remux,
            Self::Subtitles,
            Self::Thumbnail,
            Self::FFmpeg,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    Main,
    Compressor,
    Converter,
    Downloader,
    Extractor,
    Probe,
    Reformatter,
    Remux,
    Subtitles,
    Thumbnail,
    FFmpeg,
}

pub struct App<'a> {
    screen: Screen,
    selected: usize,

    compressor: Compressor<'static>,
    converter: Converter<'a>,
    downloader: Downloader<'a>,
    extractor: Extractor<'a>,
    probe: Probe<'a>,
    reformatter: Reformatter,
    remux: Remux,
    subtitles: Subtitles<'static>,
    thumbnail: Thumbnail,
    ff_family: FFFamily,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            screen: Screen::Main,
            selected: 0,

            compressor: Compressor::new(),
            converter: Converter::new(),
            downloader: Downloader::new(),
            extractor: Extractor::new(),
            probe: Probe::new(),
            reformatter: Reformatter::new(),
            remux: Remux::new(),
            subtitles: Subtitles::new(),
            thumbnail: Thumbnail::new(),
            ff_family: FFFamily::detect(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = if self.compressor.is_compressing() || self.converter.is_converting() {
                std::time::Duration::from_millis(100)
            } else {
                std::time::Duration::MAX
            };

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // Windows terminals can report Press, Repeat, and Release events.
                    // Handle only Press so one physical key does not trigger twice.
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    if !self.handle_key(key) {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.screen {
            Screen::Main => self.handle_main_key(key),
            Screen::Compressor => self.handle_compressor_key(key),
            Screen::Converter => self.handle_converter_key(key),
            Screen::Downloader => self.handle_downloader_key(key),
            Screen::Extractor => self.handle_extractor_key(key),
            Screen::Probe => self.handle_probe_key(key),
            Screen::Reformatter => self.handle_reformatter_key(key),
            Screen::Remux => self.handle_remux_key(key),
            Screen::Subtitles => self.handle_subtitles_key(key),
            Screen::Thumbnail => self.handle_thumbnail_key(key),
            Screen::FFmpeg => self.handle_ffmpeg_key(key),
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.next(),

            KeyCode::Up | KeyCode::Char('k') => self.previous(),

            KeyCode::Enter => self.open_selected(),

            KeyCode::Char('q') => return false,

            _ => {}
        }

        true
    }

    fn next(&mut self) {
        let count = MenuItem::all().len();
        self.selected = (self.selected + 1) % count;
    }

    fn previous(&mut self) {
        let count = MenuItem::all().len();
        self.selected = (self.selected + count - 1) % count;
    }

    fn open_selected(&mut self) {
        let item = MenuItem::all()[self.selected];

        self.screen = match item {
            MenuItem::Compressor => Screen::Compressor,
            MenuItem::Converter => Screen::Converter,
            MenuItem::Downloader => Screen::Downloader,
            MenuItem::Extractor => Screen::Extractor,
            MenuItem::Probe => Screen::Probe,
            MenuItem::Reformatter => Screen::Reformatter,
            MenuItem::Remux => Screen::Remux,
            MenuItem::Subtitles => Screen::Subtitles,
            MenuItem::Thumbnail => Screen::Thumbnail,
            MenuItem::FFmpeg => Screen::FFmpeg,
        };
    }

    fn handle_tool_action_missing_ffmpeg(&mut self) -> bool {
        if self.ff_family.is_available() {
            return true;
        }

        self.screen = Screen::FFmpeg;
        self.selected = MenuItem::FFmpeg.index();

        false
    }

    fn handle_compressor_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Compressor.index();
            }

            KeyCode::Char('q') => return false,

            KeyCode::Enter => {
                if !self.handle_tool_action_missing_ffmpeg() {
                    return true;
                }

                self.compressor.handle_key(key);
            }

            _ => {
                self.compressor.handle_key(key);
            }
        }

        true
    }

    fn handle_converter_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Converter.index();
            }

            KeyCode::Char('q') => return false,

            KeyCode::Enter => {
                if !self.handle_tool_action_missing_ffmpeg() {
                    return true;
                }

                self.converter.handle_key(key);
            }

            _ => {
                self.converter.handle_key(key);
            }
        }

        true
    }

    fn handle_downloader_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Downloader.index();
            }

            KeyCode::Char('q') => return false,

            KeyCode::Enter => {
                if !self.handle_tool_action_missing_ffmpeg() {
                    return true;
                }

                self.downloader.handle_key(key);
            }

            _ => {
                self.downloader.handle_key(key);
            }
        }

        true
    }

    fn handle_extractor_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Extractor.index();
            }

            KeyCode::Char('q') => return false,

            KeyCode::Enter => {
                if !self.handle_tool_action_missing_ffmpeg() {
                    return true;
                }

                self.extractor.handle_key(key);
            }

            _ => {
                self.extractor.handle_key(key);
            }
        }

        true
    }

    fn handle_probe_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Probe.index();
            }

            KeyCode::Char('q') => return false,

            _ => {
                /*
                 * Probe uses KeyEvent because it contains a TextArea.
                 *
                 * This is different from the older Probe implementation
                 * that accepted KeyCode.
                 */
                if !self.ff_family.is_available() {
                    self.screen = Screen::FFmpeg;
                    self.selected = MenuItem::FFmpeg.index();
                    return true;
                }

                self.probe.handle_key(key);
            }
        }

        true
    }

    fn handle_reformatter_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Reformatter.index();
            }

            KeyCode::Char('q') => return false,

            KeyCode::Enter => {
                if !self.handle_tool_action_missing_ffmpeg() {
                    return true;
                }

                self.reformatter.handle_key(key.code);
            }

            _ => {
                self.reformatter.handle_key(key.code);
            }
        }

        true
    }

    fn handle_remux_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Remux.index();
            }

            KeyCode::Char('q') => return false,

            KeyCode::Enter => {
                if !self.handle_tool_action_missing_ffmpeg() {
                    return true;
                }

                self.remux.handle_key(key.code);
            }

            _ => {
                self.remux.handle_key(key.code);
            }
        }

        true
    }

    fn handle_subtitles_key(&mut self, key: KeyEvent) -> bool {
        /*
         * IMPORTANT:
         *
         * When TextArea is active, Subtitles owns ALL keyboard input.
         * Do not intercept Esc, Enter, q, etc. here.
         */
        if self.subtitles.file_editing() {
            self.subtitles.handle_key(key);
            return true;
        }

        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Subtitles.index();
            }

            KeyCode::Char('q') => return false,

            _ => {
                self.subtitles.handle_key(key);
            }
        }

        true
    }

    fn handle_thumbnail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::Thumbnail.index();
            }

            KeyCode::Char('q') => return false,

            KeyCode::Enter => {
                if !self.handle_tool_action_missing_ffmpeg() {
                    return true;
                }

                self.thumbnail.handle_key(key.code);
            }

            _ => {
                self.thumbnail.handle_key(key.code);
            }
        }

        true
    }

    fn handle_ffmpeg_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.selected = MenuItem::FFmpeg.index();
            }

            KeyCode::Char('q') => return false,

            KeyCode::Enter => {
                if self.ff_family.is_available() {
                    self.screen = Screen::Main;
                    self.selected = MenuItem::FFmpeg.index();
                    return true;
                }

                // TODO:
                // Download the latest FFmpeg build
                // for the current platform.
            }

            _ => {}
        }

        true
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match self.screen {
            Screen::Main => self.draw_main(frame),

            Screen::Compressor => {
                self.compressor.draw(frame, area);
            }

            Screen::Converter => {
                self.converter.draw(frame, area);
            }

            Screen::Downloader => {
                self.downloader.draw(frame, area);
            }

            Screen::Extractor => {
                self.extractor.draw(frame, area);
            }

            Screen::Probe => {
                self.probe.draw(frame, area);
            }

            Screen::Reformatter => {
                self.reformatter.draw(frame, area);
            }

            Screen::Remux => {
                self.remux.draw(frame, area);
            }

            Screen::Subtitles => {
                self.subtitles.draw(frame, area);
            }

            Screen::Thumbnail => {
                self.thumbnail.draw(frame, area);
            }

            Screen::FFmpeg => {
                self.draw_ffmpeg(frame);
            }
        }
    }

    fn draw_main(&self, frame: &mut Frame) {
        let areas = Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(frame.area());

        self.draw_logo(frame, areas[0]);
        self.draw_tools(frame, areas[1]);
    }

    fn draw_logo(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered();
        let inner = block.inner(area);

        frame.render_widget(block, area);

        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .split(inner);

        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(80),
            Constraint::Fill(1),
        ])
        .split(vertical[1]);

        frame.render_widget(Paragraph::new(LOGO), horizontal[1]);
    }

    fn draw_tools(&self, frame: &mut Frame, area: Rect) {
        let items = MenuItem::all()
            .into_iter()
            .map(|item| ListItem::new(item.title()))
            .collect::<Vec<_>>();

        let menu = List::new(items)
            .block(Block::bordered())
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut state = ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(menu, area, &mut state);
    }

    fn draw_ffmpeg(&self, frame: &mut Frame) {
        let mut items = vec![
            format!(
                "FFprobe: {} [{}]",
                self.ff_family.ffprobe.display(),
                self.ff_family.status_for("ffprobe")
            ),
            format!(
                "FFmpeg: {} [{}]",
                self.ff_family.ffmpeg.display(),
                self.ff_family.status_for("ffmpeg")
            ),
            format!(
                "FFplay: {} [{}]",
                self.ff_family.ffplay.display(),
                self.ff_family.status_for("ffplay")
            ),
        ];

        let mut selected = None;

        if !self.ff_family.is_available() {
            items.push("Get FFmpeg binary for this platform".to_string());
            selected = Some(items.len() - 1);
        }

        let list = List::new(items.into_iter().map(ListItem::new).collect::<Vec<_>>())
            .block(Block::bordered().title("FFmpeg Troubleshooting"))
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut state = ListState::default();
        state.select(selected);

        frame.render_stateful_widget(list, frame.area(), &mut state);
    }
}
