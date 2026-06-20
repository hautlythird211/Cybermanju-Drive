pub mod duckduckgo;

#[cfg(not(target_arch = "wasm32"))]
pub mod tantivy;

use crate::WebResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl SearchQuery {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            limit: None,
            offset: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<WebResult>,
    pub total_estimate: Option<usize>,
    pub source: SearchSourceType,
    pub query: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchSourceType {
    DuckDuckGo,
    Tantivy,
    Hybrid,
    Error,
}

#[async_trait::async_trait]
pub trait SearchEngine: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SearchError>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum SearchError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Not found")]
    NotFound,
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<reqwest::Error> for SearchError {
    fn from(e: reqwest::Error) -> Self {
        SearchError::Http(e.to_string())
    }
}
