use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[tauri::command]
pub async fn web_search(
    query: String,
    limit: Option<usize>,
) -> Result<Vec<WebSearchResult>, String> {
    let limit = limit.unwrap_or(20);

    let client = reqwest::Client::builder()
        .user_agent("Cybermanju/1.0 (futuristic-search)")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let encoded_query = url::form_urlencoded::byte_serialize(query.as_bytes());
    let url = format!("https://lite.duckduckgo.com/lite/?q={}", encoded_query);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let results = parse_duckduckgo_lite(&body, limit);
    Ok(results)
}

fn parse_duckduckgo_lite(html: &str, limit: usize) -> Vec<WebSearchResult> {
    use scraper::{Html, Selector};
    let document = Html::parse_document(html);
    let table_sel = Selector::parse("table.result").expect("Invalid selector: table.result");
    let link_sel = Selector::parse("a.result-link").expect("Invalid selector: a.result-link");
    let snippet_sel =
        Selector::parse("td.result-snippet").expect("Invalid selector: td.result-snippet");

    let mut results = Vec::new();

    for row in document.select(&table_sel) {
        let title = row
            .select(&link_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let url = row
            .select(&link_sel)
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
            results.push(WebSearchResult {
                title,
                url,
                snippet,
            });
        }

        if results.len() >= limit {
            break;
        }
    }

    results
}

#[tauri::command]
pub async fn fetch_page(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("Cybermanju/1.0 (futuristic-search)")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch page: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read page: {}", e))?;

    Ok(body)
}
