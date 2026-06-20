use crate::backends::create_backend;
use crate::harvest;
use crate::portable;
use crate::transfer;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use cybermanju_types::sync::SyncBackendType;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Gauge, List, ListItem, ListState, Paragraph, Tabs,
};
use ratatui::Frame;
use serde::{Deserialize, Serialize};
use std::io::stdout;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

const APP_DIR: &str = ".cybermanju-cli";

fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR)
}

fn config_path() -> PathBuf {
    data_dir().join("backends.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredBackend {
    pub name: String,
    pub backend_type: SyncBackendType,
    pub token: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub backends: Vec<StoredBackend>,
}

impl BackendConfig {
    pub fn load() -> Self {
        let p = config_path();
        if p.exists() {
            std::fs::read_to_string(&p)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self { backends: vec![] }
        }
    }

    pub fn save(&self) {
        if let Some(parent) = config_path().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(config_path(), &data);
        }
    }
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self { backends: vec![] }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Menu,
    BackendList,
    BackendAdd,
    Harvest {
        backends: Vec<HarvestBackend>,
        overall_progress: f64,
        status_line: String,
        running: bool,
        id: u64,
        output_path: String,
    },
    Transfer {
        source_idx: usize,
        dest_idx: usize,
        status: String,
        progress: f64,
        running: bool,
    },
    Portable {
        mode: PortableMode,
        status: String,
        path: String,
        progress: f64,
        running: bool,
    },
}

#[derive(Debug, Clone)]
pub struct HarvestBackend {
    pub name: String,
    pub files_found: usize,
    pub files_downloaded: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortableMode {
    Create,
    Extract,
    List,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskMessage {
    HarvestProgress(u64, String, usize, usize, u64, String), // seq, backend_name, files_done, total_files, bytes, status
    HarvestDone(u64, String, usize, u64),
    HarvestError(u64, String, String),
    HarvestOverall(u64, f64, String),
    HarvestComplete(u64, usize, u64),
    TransferProgress(f64, String),
    TransferDone,
    TransferError(String),
    PortableProgress(f64, String),
    PortableDone(String),
    PortableError(String),
}

pub struct App {
    pub screen: Screen,
    pub backends: BackendConfig,
    pub backend_list_state: ListState,
    pub menu_index: usize,
    pub quit: bool,
    pub task_rx: Receiver<TaskMessage>,
    task_tx: Sender<TaskMessage>,
    pub status_log: Vec<String>,
    backend_add_state: BackendAddState,
    tab_index: usize,
    harvest_detail_idx: usize,
    harvest_seq: u64,
}

struct BackendAddState {
    backend_type: usize,
    name: String,
    token: String,
    config: String,
    field_focus: usize,
    error: String,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let backends = BackendConfig::load();
        let mut s = Self {
            screen: Screen::Menu,
            backends,
            backend_list_state: ListState::default(),
            menu_index: 0,
            quit: false,
            task_rx: rx,
            task_tx: tx,
            status_log: vec![],
            backend_add_state: BackendAddState {
                backend_type: 0,
                name: String::new(),
                token: String::new(),
                config: String::new(),
                field_focus: 0,
                error: String::new(),
            },
            tab_index: 0,
            harvest_detail_idx: 0,
            harvest_seq: 0,
        };
        s.backend_list_state.select(Some(0));
        s
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stderr = stdout();
        execute!(stderr, EnterAlternateScreen)?;
        let mut terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stderr))?;
        terminal.clear()?;

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        while !self.quit {
            terminal.draw(|f| self.render(f))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code);
                    }
                }
            }
            self.check_tasks();

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn check_tasks(&mut self) {
        while let Ok(msg) = self.task_rx.try_recv() {
            match msg {
                TaskMessage::HarvestProgress(seq, name, files, total, bytes, status) => {
                    if let Screen::Harvest {
                        ref mut backends,
                        id,
                        ..
                    } = self.screen
                    {
                        if seq != id {
                            continue;
                        }
                        for b in backends.iter_mut() {
                            if b.name == name {
                                b.files_downloaded = files;
                                b.files_found = total;
                                b.downloaded_bytes = bytes;
                                b.status = status.clone();
                            }
                        }
                    }
                }
                TaskMessage::HarvestDone(seq, name, files, bytes) => {
                    if let Screen::Harvest {
                        ref mut backends,
                        id,
                        ..
                    } = self.screen
                    {
                        if seq != id {
                            continue;
                        }
                        for b in backends.iter_mut() {
                            if b.name == name {
                                b.status = "done".into();
                                b.files_downloaded = files;
                                b.downloaded_bytes = bytes;
                            }
                        }
                    }
                }
                TaskMessage::HarvestError(seq, name, err) => {
                    if let Screen::Harvest { id, .. } = self.screen {
                        if seq != id {
                            continue;
                        }
                    }
                    self.status_log
                        .push(format!("{} harvest error: {}", name, err));
                    if let Screen::Harvest {
                        ref mut backends, ..
                    } = self.screen
                    {
                        for b in backends.iter_mut() {
                            if b.name == name {
                                b.status = format!("error: {}", err);
                            }
                        }
                    }
                }
                TaskMessage::HarvestOverall(seq, p, s) => {
                    if let Screen::Harvest {
                        ref mut overall_progress,
                        ref mut status_line,
                        id,
                        ..
                    } = self.screen
                    {
                        if seq != id {
                            continue;
                        }
                        *overall_progress = p;
                        *status_line = s;
                    }
                }
                TaskMessage::HarvestComplete(seq, files, bytes) => {
                    if let Screen::Harvest {
                        ref mut running,
                        ref mut status_line,
                        id,
                        ..
                    } = self.screen
                    {
                        if seq != id {
                            continue;
                        }
                        *running = false;
                        *status_line = format!("Done — {} files, {} bytes harvested", files, bytes);
                    }
                }
                TaskMessage::TransferProgress(p, s) => {
                    if let Screen::Transfer {
                        ref mut progress,
                        ref mut status,
                        ..
                    } = self.screen
                    {
                        *progress = p;
                        *status = s;
                    }
                }
                TaskMessage::TransferDone => {
                    if let Screen::Transfer {
                        ref mut running,
                        ref mut status,
                        ..
                    } = self.screen
                    {
                        *running = false;
                        *status = "Transfer complete!".into();
                    }
                }
                TaskMessage::TransferError(e) => {
                    if let Screen::Transfer {
                        ref mut running,
                        ref mut status,
                        ..
                    } = self.screen
                    {
                        *running = false;
                        *status = format!("Error: {}", e);
                    }
                }
                TaskMessage::PortableProgress(p, s) => {
                    if let Screen::Portable {
                        ref mut progress,
                        ref mut status,
                        ..
                    } = self.screen
                    {
                        *progress = p;
                        *status = s;
                    }
                }
                TaskMessage::PortableDone(s) => {
                    if let Screen::Portable {
                        ref mut running,
                        ref mut status,
                        ..
                    } = self.screen
                    {
                        *running = false;
                        *status = s;
                    }
                }
                TaskMessage::PortableError(e) => {
                    if let Screen::Portable {
                        ref mut running,
                        ref mut status,
                        ..
                    } = self.screen
                    {
                        *running = false;
                        *status = format!("Error: {}", e);
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        match self.screen.clone() {
            Screen::Menu => self.handle_menu_key(key),
            Screen::BackendList => self.handle_backend_list_key(key),
            Screen::BackendAdd { .. } => self.handle_backend_add_key(key),
            Screen::Harvest { running, .. } => {
                if !running {
                    self.handle_harvest_key(key);
                }
            }
            Screen::Transfer { running, .. } => {
                if !running {
                    self.handle_transfer_key(key);
                }
            }
            Screen::Portable { running, .. } => {
                if !running {
                    self.handle_portable_key(key);
                }
            }
        }
    }

    const MENU_ITEMS: &[&str] = &[
        "  Backends  ",
        "  Harvest   ",
        "  Transfer  ",
        "  .cybermanju Create",
        "  .cybermanju Extract",
        "  Quit      ",
    ];

    fn handle_menu_key(&mut self, key: KeyCode) {
        let max = Self::MENU_ITEMS.len() - 1;
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu_index = self.menu_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu_index = self.menu_index.saturating_add(1).min(max);
            }
            KeyCode::Enter => match self.menu_index {
                0 => self.screen = Screen::BackendList,
                1 => self.start_harvest(),
                2 => self.start_transfer(),
                3 => {
                    self.screen = Screen::Portable {
                        mode: PortableMode::Create,
                        status: String::new(),
                        path: String::new(),
                        progress: 0.0,
                        running: false,
                    }
                }
                4 => {
                    self.screen = Screen::Portable {
                        mode: PortableMode::Extract,
                        status: String::new(),
                        path: String::new(),
                        progress: 0.0,
                        running: false,
                    }
                }
                5 => self.quit = true,
                _ => {}
            },
            KeyCode::Char('q') => self.quit = true,
            _ => {}
        }
    }

    fn handle_backend_list_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.backend_list_state.selected().unwrap_or(0);
                self.backend_list_state.select(Some(i.saturating_sub(1)));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.backend_list_state.selected().unwrap_or(0);
                let max = self.backends.backends.len().saturating_sub(1);
                self.backend_list_state.select(Some((i + 1).min(max)));
            }
            KeyCode::Char('a') => {
                self.backend_add_state = BackendAddState {
                    backend_type: 0,
                    name: String::new(),
                    token: String::new(),
                    config: String::new(),
                    field_focus: 0,
                    error: String::new(),
                };
                self.screen = Screen::BackendAdd;
            }
            KeyCode::Char('d') => {
                if let Some(i) = self.backend_list_state.selected() {
                    if i < self.backends.backends.len() {
                        self.backends.backends.remove(i);
                        self.backends.save();
                        let max = self.backends.backends.len().saturating_sub(1);
                        self.backend_list_state.select(Some(i.min(max)));
                    }
                }
            }
            KeyCode::Esc => self.screen = Screen::Menu,
            _ => {}
        }
    }

    const BACKEND_TYPES: &[&str] = &[
        "local",
        "github",
        "gitlab",
        "googleDrive",
        "googlePhotos",
        "telegram",
        "mega",
    ];

    fn handle_backend_add_key(&mut self, key: KeyCode) {
        let types = Self::BACKEND_TYPES;
        let mut state = self.backend_add_state.clone();
        match key {
            KeyCode::Tab => {
                state.field_focus = (state.field_focus + 1) % 5;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if state.field_focus == 0 {
                    state.backend_type = state.backend_type.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if state.field_focus == 0 {
                    state.backend_type =
                        (state.backend_type + 1).min(Self::BACKEND_TYPES.len() - 1);
                }
            }
            KeyCode::Enter => {
                if state.field_focus == 4 {
                    if state.name.is_empty() || state.token.is_empty() {
                        state.error = "Name and token are required".into();
                    } else {
                        let bt: SyncBackendType = match types[state.backend_type] {
                            "local" => SyncBackendType::Local,
                            "github" => SyncBackendType::GitHub,
                            "gitlab" => SyncBackendType::GitLab,
                            "googleDrive" => SyncBackendType::GoogleDrive,
                            "googlePhotos" => SyncBackendType::GooglePhotos,
                            "telegram" => SyncBackendType::Telegram,
                            "mega" => SyncBackendType::Mega,
                            _ => SyncBackendType::Local,
                        };
                        let config: serde_json::Value = if state.config.is_empty() {
                            serde_json::json!({})
                        } else {
                            serde_json::from_str(&state.config)
                                .unwrap_or_else(|_| serde_json::json!({}))
                        };
                        self.backends.backends.push(StoredBackend {
                            name: state.name.clone(),
                            backend_type: bt,
                            token: state.token.clone(),
                            config,
                        });
                        self.backends.save();
                        self.screen = Screen::BackendList;
                        return;
                    }
                } else {
                    state.field_focus = (state.field_focus + 1) % 5;
                }
            }
            KeyCode::Char(c) => match state.field_focus {
                1 => state.name.push(c),
                2 => state.token.push(c),
                3 => state.config.push(c),
                _ => {}
            },
            KeyCode::Backspace => match state.field_focus {
                1 => {
                    state.name.pop();
                }
                2 => {
                    state.token.pop();
                }
                3 => {
                    state.config.pop();
                }
                _ => {}
            },
            KeyCode::Esc => {
                self.screen = Screen::BackendList;
                return;
            }
            _ => {}
        }
        self.backend_add_state = state;
        self.screen = Screen::BackendAdd;
    }

    fn handle_harvest_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.harvest_detail_idx = self.harvest_detail_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = if let Screen::Harvest { ref backends, .. } = self.screen {
                    backends.len().saturating_sub(1)
                } else {
                    0
                };
                self.harvest_detail_idx = (self.harvest_detail_idx + 1).min(max);
            }
            KeyCode::Char('h') => self.start_harvest(),
            KeyCode::Char(c) => {
                if let Screen::Harvest {
                    ref mut output_path,
                    ..
                } = self.screen
                {
                    output_path.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Screen::Harvest {
                    ref mut output_path,
                    ..
                } = self.screen
                {
                    output_path.pop();
                }
            }
            KeyCode::Esc => self.screen = Screen::Menu,
            _ => {}
        }
    }

    fn handle_transfer_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if let Screen::Transfer {
                    ref mut source_idx, ..
                } = self.screen
                {
                    *source_idx = source_idx.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Screen::Transfer {
                    ref mut source_idx, ..
                } = self.screen
                {
                    let max = self.backends.backends.len().saturating_sub(1);
                    *source_idx = (*source_idx + 1).min(max);
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let Screen::Transfer {
                    ref mut dest_idx, ..
                } = self.screen
                {
                    *dest_idx = dest_idx.saturating_sub(1);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Screen::Transfer {
                    ref mut dest_idx, ..
                } = self.screen
                {
                    let max = self.backends.backends.len().saturating_sub(1);
                    *dest_idx = (*dest_idx + 1).min(max);
                }
            }
            KeyCode::Char('t') => self.start_transfer(),
            KeyCode::Esc => self.screen = Screen::Menu,
            _ => {}
        }
    }

    fn handle_portable_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('c') => {
                self.screen = Screen::Portable {
                    mode: PortableMode::Create,
                    status: String::new(),
                    path: String::new(),
                    progress: 0.0,
                    running: false,
                };
            }
            KeyCode::Char('x') => {
                self.screen = Screen::Portable {
                    mode: PortableMode::Extract,
                    status: String::new(),
                    path: String::new(),
                    progress: 0.0,
                    running: false,
                };
            }
            KeyCode::Char('l') => {
                self.screen = Screen::Portable {
                    mode: PortableMode::List,
                    status: String::new(),
                    path: String::new(),
                    progress: 0.0,
                    running: false,
                };
            }
            KeyCode::Enter => {
                if let Screen::Portable {
                    ref mode,
                    ref path,
                    running,
                    ..
                } = self.screen
                {
                    if !running {
                        self.run_portable_action(mode.clone(), path.clone());
                    }
                }
            }
            KeyCode::Char(c) => {
                if let Screen::Portable { ref mut path, .. } = self.screen {
                    path.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Screen::Portable { ref mut path, .. } = self.screen {
                    path.pop();
                }
            }
            KeyCode::Esc => self.screen = Screen::Menu,
            _ => {}
        }
    }

    fn start_harvest(&mut self) {
        let tx = self.task_tx.clone();
        let backends = self.backends.backends.clone();
        if backends.is_empty() {
            return;
        }
        self.harvest_seq += 1;
        let seq = self.harvest_seq;
        let output_path = if let Screen::Harvest {
            ref output_path, ..
        } = self.screen
        {
            if output_path.is_empty() {
                None
            } else {
                Some(output_path.clone())
            }
        } else {
            None
        };
        let hb: Vec<HarvestBackend> = backends
            .iter()
            .map(|b| HarvestBackend {
                name: b.name.clone(),
                files_found: 0,
                files_downloaded: 0,
                total_bytes: 0,
                downloaded_bytes: 0,
                status: "waiting".into(),
            })
            .collect();
        self.screen = Screen::Harvest {
            backends: hb,
            overall_progress: 0.0,
            status_line: "Starting harvest...".into(),
            running: true,
            id: seq,
            output_path: output_path.clone().unwrap_or_default(),
        };
        self.harvest_detail_idx = 0;
        let out_str = output_path;
        std::thread::spawn(move || {
            harvest::run_harvest_with_output(backends, tx, out_str, seq);
        });
    }

    fn start_transfer(&mut self) {
        let backends = self.backends.backends.clone();
        let idx = if let Screen::Transfer {
            source_idx,
            dest_idx,
            ..
        } = self.screen.clone()
        {
            (source_idx, dest_idx)
        } else {
            (0, 1.min(backends.len().saturating_sub(1)))
        };
        if idx.0 == idx.1 || backends.is_empty() {
            return;
        }
        let tx = self.task_tx.clone();
        self.screen = Screen::Transfer {
            source_idx: idx.0,
            dest_idx: idx.1,
            status: "Transferring...".into(),
            progress: 0.0,
            running: true,
        };
        std::thread::spawn(move || {
            transfer::run_transfer(backends, idx.0, idx.1, tx);
        });
    }

    fn run_portable_action(&mut self, mode: PortableMode, path: String) {
        if path.is_empty() {
            return;
        }
        let tx = self.task_tx.clone();
        let p = path.clone();
        self.screen = Screen::Portable {
            status: "Working...".into(),
            mode: mode.clone(),
            path,
            progress: 0.0,
            running: true,
        };
        std::thread::spawn(move || match mode {
            PortableMode::Create => portable::create_archive(&p, tx),
            PortableMode::Extract => portable::extract_archive(&p, tx),
            PortableMode::List => portable::list_archive(&p, tx),
        });
    }

    fn render(&self, f: &mut Frame) {
        match self.screen.clone() {
            Screen::Menu => self.render_menu(f),
            Screen::BackendList => self.render_backend_list(f),
            Screen::BackendAdd { .. } => self.render_backend_add(f),
            Screen::Harvest { .. } => self.render_harvest(f),
            Screen::Transfer { .. } => self.render_transfer(f),
            Screen::Portable { .. } => self.render_portable(f),
        }
    }

    fn render_menu(&self, f: &mut Frame) {
        let list_items: Vec<ListItem> = Self::MENU_ITEMS
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == self.menu_index {
                    Style::new()
                        .fg(Color::Rgb(0, 255, 65))
                        .bg(Color::Rgb(0, 40, 0))
                } else {
                    Style::new().fg(Color::Rgb(0, 200, 50))
                };
                ListItem::new(Line::from(Span::styled(s, style)))
            })
            .collect();
        let list = List::new(list_items)
            .block(
                Block::default()
                    .title(" Cybermanju Drive CLI ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
            )
            .highlight_style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 65))
                    .add_modifier(Modifier::BOLD),
            );
        let area = centered_rect(f.area(), 40, 50);
        f.render_widget(list, area);

        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                "  ↑/↓ navigate  Enter select  q quit  ",
                Style::new().fg(Color::Rgb(100, 100, 100)),
            ),
            Span::styled(version, Style::new().fg(Color::Rgb(60, 60, 60))),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(
            footer,
            Rect::new(area.x, area.y + area.height + 1, area.width, 1),
        );
    }

    fn render_backend_list(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(f.area());
        let items: Vec<ListItem> = self
            .backends
            .backends
            .iter()
            .map(|b| ListItem::new(format!(" {} ({})", b.name, format!("{:?}", b.backend_type))))
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Configured Backends ")
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
            )
            .highlight_style(
                Style::new()
                    .fg(Color::Rgb(0, 255, 65))
                    .bg(Color::Rgb(0, 40, 0)),
            );
        let mut state = self.backend_list_state.clone();
        f.render_stateful_widget(list, chunks[0], &mut state);
        let help = Paragraph::new(Line::from(vec![
            Span::styled("  a  add  ", Style::new().fg(Color::Rgb(0, 255, 65))),
            Span::styled("d  delete  ", Style::new().fg(Color::Rgb(255, 100, 100))),
            Span::styled("Esc  back  ", Style::new().fg(Color::Rgb(100, 100, 100))),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(help, chunks[1]);
    }

    fn render_backend_add(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(f.area());
        let types = Self::BACKEND_TYPES;
        let sel = self.backend_add_state.backend_type;
        let type_str = types[sel];
        let focused = self.backend_add_state.field_focus;
        let highlight = |i| {
            if i == focused {
                Style::new()
                    .fg(Color::Rgb(0, 255, 65))
                    .bg(Color::Rgb(0, 40, 0))
            } else {
                Style::new().fg(Color::Rgb(180, 180, 180))
            }
        };

        let type_line = Paragraph::new(Line::from(vec![
            Span::styled(" Type: ", Style::new().fg(Color::Rgb(100, 100, 100))),
            Span::styled(type_str, highlight(0)),
            Span::styled("  (↑↓)  ", Style::new().fg(Color::Rgb(80, 80, 80))),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
        );
        f.render_widget(type_line, chunks[0]);

        let name_line = Paragraph::new(Line::from(vec![
            Span::styled(" Name: ", Style::new().fg(Color::Rgb(100, 100, 100))),
            Span::styled(&self.backend_add_state.name, highlight(1)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
        );
        f.render_widget(name_line, chunks[1]);

        let token_line = Paragraph::new(Line::from(vec![
            Span::styled(" Token: ", Style::new().fg(Color::Rgb(100, 100, 100))),
            Span::styled(&self.backend_add_state.token, highlight(2)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
        );
        f.render_widget(token_line, chunks[2]);

        let config_line = Paragraph::new(Line::from(vec![
            Span::styled(" Config JSON: ", Style::new().fg(Color::Rgb(100, 100, 100))),
            Span::styled(&self.backend_add_state.config, highlight(3)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
        );
        f.render_widget(config_line, chunks[3]);

        let btns = Paragraph::new(Line::from(vec![Span::styled(
            " [Save] ",
            if focused == 4 {
                Style::new()
                    .fg(Color::Rgb(0, 255, 65))
                    .bg(Color::Rgb(0, 40, 0))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::Rgb(180, 180, 180))
            },
        )]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
        );
        f.render_widget(btns, chunks[4]);

        let err_color = if self.backend_add_state.error.is_empty() {
            Color::Rgb(60, 60, 60)
        } else {
            Color::Rgb(255, 100, 100)
        };
        let help = Paragraph::new(Line::from(vec![
            Span::styled(
                " Tab:switch  ↑↓:change type  Enter:save  Esc:cancel  ",
                Style::new().fg(err_color),
            ),
            Span::styled(
                &self.backend_add_state.error,
                Style::new().fg(Color::Rgb(255, 100, 100)),
            ),
        ]));
        f.render_widget(help, chunks[5]);
    }

    fn render_harvest(&self, f: &mut Frame) {
        if let Screen::Harvest {
            ref backends,
            overall_progress,
            ref status_line,
            running,
            ref output_path,
            ..
        } = self.screen
        {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(" Overall Progress ")
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
                )
                .gauge_style(
                    Style::new()
                        .fg(Color::Rgb(0, 255, 65))
                        .bg(Color::Rgb(0, 30, 0)),
                )
                .percent((overall_progress * 100.0) as u16)
                .label(format!(
                    "{} — {}%",
                    status_line,
                    (overall_progress * 100.0) as u16
                ));
            f.render_widget(gauge, chunks[0]);

            let out_style = if running {
                Style::new().fg(Color::Rgb(100, 100, 100))
            } else {
                Style::new().fg(Color::Rgb(0, 255, 65))
            };
            let out_line = Paragraph::new(Line::from(vec![
                Span::styled(" Output: ", Style::new().fg(Color::Rgb(100, 100, 100))),
                Span::styled(
                    if output_path.is_empty() {
                        "(auto)"
                    } else {
                        output_path.as_str()
                    },
                    out_style,
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
            );
            f.render_widget(out_line, chunks[1]);

            let items: Vec<ListItem> = backends
                .iter()
                .map(|b| {
                    let color = match b.status.as_str() {
                        "done" => Color::Rgb(0, 200, 100),
                        "error" => Color::Rgb(255, 100, 100),
                        "waiting" => Color::Rgb(100, 100, 100),
                        _ => Color::Rgb(0, 255, 65),
                    };
                    let pct = if b.files_found > 0 {
                        (b.files_downloaded as f64 / b.files_found as f64 * 100.0) as u16
                    } else {
                        0
                    };
                    ListItem::new(format!(
                        " {:<20} {:>4}/{} files {:>8} bytes {:>3}%  {}",
                        b.name,
                        b.files_downloaded,
                        b.files_found,
                        b.downloaded_bytes,
                        pct,
                        b.status
                    ))
                    .style(Style::new().fg(color))
                })
                .collect();
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" Backends ")
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
                )
                .highlight_style(
                    Style::new()
                        .fg(Color::Rgb(0, 255, 65))
                        .bg(Color::Rgb(0, 40, 0)),
                );
            let mut state = ListState::default().with_selected(Some(self.harvest_detail_idx));
            f.render_stateful_widget(list, chunks[2], &mut state);

            let help = Paragraph::new(Line::from(vec![
                Span::styled(" h restart  ", Style::new().fg(Color::Rgb(0, 255, 65))),
                Span::styled("Esc back  ", Style::new().fg(Color::Rgb(100, 100, 100))),
                if running {
                    Span::styled(" [running]", Style::new().fg(Color::Rgb(255, 200, 0)))
                } else {
                    Span::styled(" [idle]", Style::new().fg(Color::Rgb(100, 100, 100)))
                },
                Span::styled(
                    "  type output path then h  ",
                    Style::new().fg(Color::Rgb(80, 80, 80)),
                ),
            ]));
            f.render_widget(help, chunks[3]);
        }
    }

    fn render_transfer(&self, f: &mut Frame) {
        if let Screen::Transfer {
            source_idx,
            dest_idx,
            ref status,
            progress,
            running,
        } = self.screen
        {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(f.area());

            let names: Vec<&str> = self
                .backends
                .backends
                .iter()
                .map(|b| b.name.as_str())
                .collect();
            let src_name = names.get(source_idx).copied().unwrap_or("?");
            let dst_name = names.get(dest_idx).copied().unwrap_or("?");
            let info = Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Source:      ", Style::new().fg(Color::Rgb(100, 100, 100))),
                    Span::styled(
                        src_name,
                        if running {
                            Style::new().fg(Color::Rgb(100, 100, 100))
                        } else {
                            Style::new()
                                .fg(Color::Rgb(0, 255, 65))
                                .add_modifier(Modifier::BOLD)
                        },
                    ),
                ]),
                Line::from(vec![
                    Span::styled(" Destination: ", Style::new().fg(Color::Rgb(100, 100, 100))),
                    Span::styled(
                        dst_name,
                        if running {
                            Style::new().fg(Color::Rgb(100, 100, 100))
                        } else {
                            Style::new()
                                .fg(Color::Rgb(0, 255, 65))
                                .add_modifier(Modifier::BOLD)
                        },
                    ),
                ]),
            ])
            .block(
                Block::default()
                    .title(" Transfer ")
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
            );
            f.render_widget(info, chunks[0]);

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
                )
                .gauge_style(
                    Style::new()
                        .fg(Color::Rgb(0, 255, 65))
                        .bg(Color::Rgb(0, 30, 0)),
                )
                .percent((progress * 100.0) as u16)
                .label(status.as_str());
            f.render_widget(gauge, chunks[1]);

            let help = Paragraph::new(Line::from(vec![Span::styled(
                " ↑↓ source  ←→ dest  t transfer  Esc back  ",
                Style::new().fg(Color::Rgb(100, 100, 100)),
            )]));
            f.render_widget(help, chunks[2]);
        }
    }

    fn render_portable(&self, f: &mut Frame) {
        if let Screen::Portable {
            ref mode,
            ref status,
            ref path,
            progress,
            running,
        } = self.screen
        {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(f.area());

            let mode_str = match mode {
                PortableMode::Create => "Create .cybermanju archive",
                PortableMode::Extract => "Extract .cybermanju archive",
                PortableMode::List => "List .cybermanju contents",
            };
            let mode_tabs = Tabs::new(vec![" Create ", " Extract ", " List "])
                .block(
                    Block::default()
                        .title(" Mode ")
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
                )
                .select(match mode {
                    PortableMode::Create => 0,
                    PortableMode::Extract => 1,
                    PortableMode::List => 2,
                })
                .style(Style::new().fg(Color::Rgb(100, 100, 100)))
                .highlight_style(
                    Style::new()
                        .fg(Color::Rgb(0, 255, 65))
                        .add_modifier(Modifier::BOLD),
                );
            f.render_widget(mode_tabs, chunks[0]);

            let path_line = Paragraph::new(Line::from(vec![
                Span::styled(" Path: ", Style::new().fg(Color::Rgb(100, 100, 100))),
                Span::styled(path.as_str(), Style::new().fg(Color::Rgb(0, 255, 65))),
            ]))
            .block(
                Block::default()
                    .title(" File ")
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
            );
            f.render_widget(path_line, chunks[1]);

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::new().fg(Color::Rgb(0, 255, 65))),
                )
                .gauge_style(
                    Style::new()
                        .fg(Color::Rgb(0, 255, 65))
                        .bg(Color::Rgb(0, 30, 0)),
                )
                .percent((progress * 100.0) as u16)
                .label(status.as_str());
            f.render_widget(gauge, chunks[2]);

            let help = Paragraph::new(Line::from(vec![Span::styled(
                " c create  x extract  l list  Enter run  type path  Esc back  ",
                Style::new().fg(Color::Rgb(100, 100, 100)),
            )]));
            f.render_widget(help, chunks[3]);
        }
    }
}

fn centered_rect(r: Rect, w_pct: u16, h_pct: u16) -> Rect {
    let w = r.width * w_pct / 100;
    let h = r.height * h_pct / 100;
    let x = r.x + (r.width - w) / 2;
    let y = r.y + (r.height - h) / 2;
    Rect::new(x, y, w, h)
}
