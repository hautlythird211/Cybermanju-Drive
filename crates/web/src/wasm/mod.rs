use wasm_bindgen::prelude::*;
use crate::search::{duckduckgo::DuckDuckGoEngine, SearchEngine, SearchQuery};
use crate::render::HtmlRenderer;
use crate::browser::BrowserManager;
use crate::WebResult;

#[wasm_bindgen]
pub struct WasmSearch {
    engine: DuckDuckGoEngine,
}

#[wasm_bindgen]
impl WasmSearch {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { engine: DuckDuckGoEngine::new() }
    }

    pub async fn search(&self, query: &str) -> Result<JsValue, JsValue> {
        let q = SearchQuery::new(query);
        let result = self.engine.search(&q).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))?)
    }
}

#[wasm_bindgen]
pub struct WasmBrowser {
    inner: BrowserManager,
}

#[wasm_bindgen]
impl WasmBrowser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: BrowserManager::new() }
    }

    pub fn open_tab(&mut self, url: &str) -> String {
        self.inner.open_tab(url)
    }

    pub fn close_tab(&mut self, id: &str) {
        self.inner.close_tab(id);
    }

    pub fn navigate(&mut self, url: &str, title: &str) {
        self.inner.navigate(url, title);
    }

    pub fn add_bookmark(&mut self, url: &str, title: &str, folder: &str) -> String {
        self.inner.add_bookmark(url, title, folder)
    }

    pub fn tabs_json(&self) -> String {
        serde_json::to_string(&self.inner.tabs).unwrap_or_default()
    }

    pub fn history_json(&self) -> String {
        serde_json::to_string(&self.inner.history).unwrap_or_default()
    }

    pub fn bookmarks_json(&self) -> String {
        serde_json::to_string(&self.inner.bookmarks).unwrap_or_default()
    }

    pub fn active_tab_json(&self) -> String {
        self.inner.active_tab()
            .map(|t| serde_json::to_string(t).unwrap_or_default())
            .unwrap_or_default()
    }
}

#[wasm_bindgen]
pub fn fetch_page_text(html: &str) -> String {
    let renderer = HtmlRenderer::new();
    renderer.extract_text(html)
}

#[wasm_bindgen]
pub fn fetch_page_title(html: &str) -> String {
    let renderer = HtmlRenderer::new();
    renderer.extract_title(html).unwrap_or_default()
}

#[wasm_bindgen]
pub fn fetch_page_links(html: &str, base_url: &str) -> String {
    let renderer = HtmlRenderer::new();
    let links = renderer.extract_links(html, base_url);
    serde_json::to_string(&links).unwrap_or_default()
}
