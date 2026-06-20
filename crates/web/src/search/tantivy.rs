use super::{SearchEngine, SearchError, SearchQuery, SearchResponse, SearchSourceType};
use crate::WebResult;
use cybermanju_search::SearchIndex;
use std::sync::RwLock;

pub struct TantivyEngine {
    index: RwLock<SearchIndex>,
    name: String,
}

impl TantivyEngine {
    pub fn new(path: &str) -> Result<Self, anyhow::Error> {
        let index = SearchIndex::new(path)?;
        Ok(Self {
            index: RwLock::new(index),
            name: format!("Tantivy@{}", path),
        })
    }

    pub fn new_with_index(index: SearchIndex) -> Self {
        Self {
            index: RwLock::new(index),
            name: "Tantivy".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl SearchEngine for TantivyEngine {
    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SearchError> {
        let query_str = query.query.clone();
        let limit = query.limit.unwrap_or(20);

        let index = self
            .index
            .read()
            .map_err(|e| SearchError::Internal(e.to_string()))?;
        let request = cybermanju_search::SearchRequest {
            query: query_str.clone(),
            limit: Some(limit),
            offset: query.offset,
        };

        let results = index
            .search(&request)
            .map_err(|e| SearchError::Internal(e.to_string()))?;

        let web_results: Vec<WebResult> = results
            .into_iter()
            .map(|r| WebResult {
                title: r.file_name,
                url: format!("cybermanju://file/{}", r.file_id),
                snippet: r.snippet,
            })
            .collect();

        let count = web_results.len();
        Ok(SearchResponse {
            results: web_results,
            total_estimate: Some(count),
            source: SearchSourceType::Tantivy,
            query: query_str,
            suggestion: None,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
