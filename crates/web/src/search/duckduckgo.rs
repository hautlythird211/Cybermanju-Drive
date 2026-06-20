use super::{SearchEngine, SearchError, SearchQuery, SearchResponse, SearchSourceType};
use crate::WebResult;
use scraper::{Html, Selector};

pub struct DuckDuckGoEngine {
    client: reqwest::Client,
}

impl DuckDuckGoEngine {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Cybermanju/1.0 (futuristic-search)")
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::ACCEPT,
                    "text/html,application/xhtml+xml".parse().unwrap(),
                );
                h
            })
            .build()
            .unwrap_or_default();
        Self { client }
    }

    async fn search_lite(&self, query: &str) -> Result<SearchResponse, SearchError> {
        let url = format!(
            "https://lite.duckduckgo.com/lite/?q={}",
            url::form_urlencoded::byte_serialize(query.as_bytes())
        );
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;
        self.parse_lite_results(&body, query)
    }

    fn parse_lite_results(&self, html: &str, query: &str) -> Result<SearchResponse, SearchError> {
        let document = Html::parse_document(html);
        let table_sel =
            Selector::parse("table.result").map_err(|e| SearchError::Parse(e.to_string()))?;
        let link_sel =
            Selector::parse("a.result-link").map_err(|e| SearchError::Parse(e.to_string()))?;
        let snippet_sel =
            Selector::parse("td.result-snippet").map_err(|e| SearchError::Parse(e.to_string()))?;
        let url_sel =
            Selector::parse("a.result-url").map_err(|e| SearchError::Parse(e.to_string()))?;

        let mut results = Vec::new();

        for row in document.select(&table_sel) {
            let title = row
                .select(&link_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let url = row
                .select(&url_sel)
                .next()
                .and_then(|e| e.attr("href"))
                .map(|u| {
                    if u.starts_with("//") {
                        format!("https:{}", u)
                    } else if u.starts_with('/') {
                        format!("https://lite.duckduckgo.com{}", u)
                    } else {
                        u.to_string()
                    }
                })
                .unwrap_or_default();

            let snippet = row
                .select(&snippet_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if !title.is_empty() {
                results.push(WebResult {
                    title,
                    url,
                    snippet,
                });
            }

            if results.len() >= 20 {
                break;
            }
        }

        if results.is_empty() {
            let suggestion = self.extract_suggestion(&document);
            return Ok(SearchResponse {
                results: vec![],
                total_estimate: None,
                source: SearchSourceType::DuckDuckGo,
                query: query.to_string(),
                suggestion,
            });
        }

        Ok(SearchResponse {
            results,
            total_estimate: Some(results.len()),
            source: SearchSourceType::DuckDuckGo,
            query: query.to_string(),
            suggestion: None,
        })
    }

    fn extract_suggestion(&self, doc: &Html) -> Option<String> {
        let suggest_sel = Selector::parse("a.ddg-spelling").ok()?;
        doc.select(&suggest_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

impl Default for DuckDuckGoEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SearchEngine for DuckDuckGoEngine {
    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SearchError> {
        self.search_lite(&query.query).await
    }

    fn name(&self) -> &str {
        "DuckDuckGo"
    }
}
