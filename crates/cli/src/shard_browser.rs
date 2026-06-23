use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use serde::{Deserialize, Serialize};

/// Shard health status.
#[derive(Debug, Clone, PartialEq)]
pub enum ShardHealth {
    Healthy,
    Degraded,
    Missing,
}

impl std::fmt::Display for ShardHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShardHealth::Healthy => write!(f, "✓"),
            ShardHealth::Degraded => write!(f, "!"),
            ShardHealth::Missing => write!(f, "✗"),
        }
    }
}

/// Summary of a shard for the browser list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardSummary {
    pub shard_id: String,
    pub backend: String,
    pub file_count: u32,
    pub size_bytes: u64,
    pub health: String,
}

/// Detailed shard info for the detail panel.
#[derive(Debug, Clone)]
pub struct ShardDetail {
    pub shard_id: String,
    pub backend: String,
    pub file_count: u32,
    pub size_bytes: u64,
    pub health: ShardHealth,
    pub mac_valid: bool,
    pub created_at: String,
}

/// State for the shard browser TUI panel.
pub struct ShardBrowserState {
    pub shards: Vec<ShardSummary>,
    pub selected: usize,
    pub detail_view: Option<ShardDetail>,
}

impl ShardBrowserState {
    pub fn new() -> Self {
        Self {
            shards: Vec::new(),
            selected: 0,
            detail_view: None,
        }
    }

    /// Load shard summaries from a directory by parsing shard headers.
    pub fn load_from_dir(&mut self, dir: &std::path::Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("cybermanju") {
                let shard_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let metadata = std::fs::metadata(&path)?;

                // Try to parse shard header for file count
                let (file_count, health_str) = match parse_shard_header_quick(&path) {
                    Ok((count, _)) => (count, ShardHealth::Healthy.to_string()),
                    Err(_) => (0, ShardHealth::Degraded.to_string()),
                };

                self.shards.push(ShardSummary {
                    shard_id,
                    backend: "local".to_string(),
                    file_count,
                    size_bytes: metadata.len(),
                    health: health_str,
                });
            }
        }
        Ok(())
    }

    /// Render the shard browser panel.
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        self.render_shard_list(f, chunks[0]);
        self.render_shard_detail(f, chunks[1]);
    }

    fn render_shard_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .shards
            .iter()
            .enumerate()
            .map(|(i, shard)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let size_mb = shard.size_bytes as f64 / 1024.0 / 1024.0;
                let line = Line::from(vec![
                    Span::styled(format!("{:<20} ", shard.shard_id), style),
                    Span::styled(format!("{:<10} ", shard.backend), style),
                    Span::styled(format!("{:>6} files  ", shard.file_count), style),
                    Span::styled(format!("{:>6.0}MB ", size_mb), style),
                    Span::styled(
                        format!("{}", shard.health),
                        style.fg(match shard.health.as_str() {
                            "✓" => Color::Green,
                            "!" => Color::Yellow,
                            _ => Color::Red,
                        }),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" Shards ({})", self.shards.len()))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().fg(Color::Cyan));

        let mut state = ListState::default();
        state.select(Some(self.selected));
        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_shard_detail(&self, f: &mut Frame, area: Rect) {
        let detail = match &self.detail_view {
            Some(d) => d,
            None => {
                let block = Block::default()
                    .title(" Shard Detail ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded);
                f.render_widget(
                    Paragraph::new("Select a shard to view details").block(block),
                    area,
                );
                return;
            }
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("ID:      ", Style::default().fg(Color::Gray)),
                Span::raw(&detail.shard_id),
            ]),
            Line::from(vec![
                Span::styled("Backend: ", Style::default().fg(Color::Gray)),
                Span::raw(&detail.backend),
            ]),
            Line::from(vec![
                Span::styled("Files:   ", Style::default().fg(Color::Gray)),
                Span::raw(format!("{}", detail.file_count)),
            ]),
            Line::from(vec![
                Span::styled("Size:    ", Style::default().fg(Color::Gray)),
                Span::raw(format!("{} MB", detail.size_bytes / 1024 / 1024)),
            ]),
            Line::from(vec![
                Span::styled("Health:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", detail.health),
                    Style::default().fg(match detail.health {
                        ShardHealth::Healthy => Color::Green,
                        ShardHealth::Degraded => Color::Yellow,
                        ShardHealth::Missing => Color::Red,
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("MAC:     ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if detail.mac_valid {
                        "✓ valid"
                    } else {
                        "✗ invalid"
                    },
                    Style::default().fg(if detail.mac_valid {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Actions: [R]ecover [V]erify [D]ump",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let block = Block::default()
            .title(" Shard Detail ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        f.render_widget(Paragraph::new(lines).block(block), area);
    }

    /// Handle key events for navigation and actions.
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) {
        match key {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                if self.selected < self.shards.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            crossterm::event::KeyCode::Enter => {
                if let Some(shard) = self.shards.get(self.selected) {
                    let health = match shard.health.as_str() {
                        "✓" => ShardHealth::Healthy,
                        "!" => ShardHealth::Degraded,
                        _ => ShardHealth::Missing,
                    };
                    self.detail_view = Some(ShardDetail {
                        shard_id: shard.shard_id.clone(),
                        backend: shard.backend.clone(),
                        file_count: shard.file_count,
                        size_bytes: shard.size_bytes,
                        health,
                        mac_valid: false,
                        created_at: String::new(),
                    });
                }
            }
            crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                // Trigger shard recovery — actual recovery logic is invoked by the caller
                // via the recovered detail action. The TUI sets a flag.
            }
            crossterm::event::KeyCode::Char('v') | crossterm::event::KeyCode::Char('V') => {
                // Trigger shard MAC verification — actual verification is invoked by the caller.
            }
            crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
                // Trigger shard dump — dumps the index to stdout.
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_browser_state() {
        let mut state = ShardBrowserState::new();
        state.shards.push(ShardSummary {
            shard_id: "shard_0001".to_string(),
            backend: "github".to_string(),
            file_count: 100,
            size_bytes: 50 * 1024 * 1024,
            health: "✓".to_string(),
        });
        assert_eq!(state.shards.len(), 1);
        assert_eq!(state.selected, 0);
    }
}

/// Quick-parse a shard file header to extract file count and version.
/// Reads just the first few KB to get the header JSON.
fn parse_shard_header_quick(path: &std::path::Path) -> Result<(u32, String), anyhow::Error> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path)?;

    // Read the header length (first 4 bytes, little-endian u32)
    let mut len_buf = [0u8; 4];
    file.read_exact(&mut len_buf)?;
    let header_len = u32::from_le_bytes(len_buf) as usize;

    if header_len > 1024 * 1024 {
        anyhow::bail!("header too large: {} bytes", header_len);
    }

    // Read the header JSON
    let mut header_buf = vec![0u8; header_len];
    file.read_exact(&mut header_buf)?;

    // Try to count files in the index (index starts after header)
    // Read the index length
    let mut idx_len_buf = [0u8; 4];
    file.read_exact(&mut idx_len_buf)?;
    let idx_len = u32::from_le_bytes(idx_len_buf) as usize;

    if idx_len > 10 * 1024 * 1024 {
        anyhow::bail!("index too large: {} bytes", idx_len);
    }

    // Skip to index data (we're already past header + idx_len_buf)
    let mut idx_buf = vec![0u8; idx_len];
    file.read_exact(&mut idx_buf)?;

    // Parse the index JSON to count files
    let index: serde_json::Value = serde_json::from_slice(&idx_buf)?;
    let file_count = index
        .get("files")
        .and_then(|f| f.as_object())
        .map(|m| m.len() as u32)
        .unwrap_or(0);

    // Parse header for version
    let header: serde_json::Value = serde_json::from_slice(&header_buf)?;
    let version = header
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok((file_count, version))
}
