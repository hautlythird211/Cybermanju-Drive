pub mod browser;
pub mod render;
pub mod search;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use browser::{Bookmark, BrowserManager, HistoryEntry, Tab};
pub use render::HtmlRenderer;
pub use search::SearchEngine;

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
