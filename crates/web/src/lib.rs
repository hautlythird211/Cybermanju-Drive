pub mod search;
pub mod browser;
pub mod render;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use search::SearchEngine;
pub use browser::{BrowserManager, Tab, HistoryEntry, Bookmark};
pub use render::HtmlRenderer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchSource {
    Local,
    Web,
    Hybrid,
}
