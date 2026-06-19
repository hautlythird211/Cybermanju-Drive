mod manager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use manager::BrowserManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub loading: bool,
    pub history_index: usize,
}

impl Tab {
    pub fn new(url: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: url.to_string(),
            url: url.to_string(),
            loading: false,
            history_index: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: String,
    pub timestamp: i64,
    pub visit_count: u32,
}

impl HistoryEntry {
    pub fn new(url: &str, title: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url: url.to_string(),
            title: title.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            visit_count: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub url: String,
    pub title: String,
    pub folder: String,
    pub created_at: i64,
}

impl Bookmark {
    pub fn new(url: &str, title: &str, folder: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url: url.to_string(),
            title: title.to_string(),
            folder: folder.to_string(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    pub home_page: String,
    pub search_engine: String,
    pub enable_js: bool,
    pub dark_mode: bool,
    pub privacy_mode: bool,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            home_page: "https://lite.duckduckgo.com".to_string(),
            search_engine: "DuckDuckGo".to_string(),
            enable_js: false,
            dark_mode: true,
            privacy_mode: true,
        }
    }
}
