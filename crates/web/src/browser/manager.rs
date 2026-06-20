use super::{Bookmark, BrowserSettings, HistoryEntry, Tab};
use std::collections::HashMap;

pub struct BrowserManager {
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<String>,
    pub history: Vec<HistoryEntry>,
    pub bookmarks: Vec<Bookmark>,
    pub settings: BrowserSettings,
    tab_counter: u64,
    history_map: HashMap<String, HistoryEntry>,
    bookmark_folders: Vec<String>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            history: Vec::new(),
            bookmarks: Vec::new(),
            settings: BrowserSettings::default(),
            tab_counter: 0,
            history_map: HashMap::new(),
            bookmark_folders: vec![
                "Root".to_string(),
                "Search".to_string(),
                "Social".to_string(),
            ],
        }
    }

    pub fn open_tab(&mut self, url: &str) -> String {
        let mut tab = Tab::new(url);
        tab.id = format!("tab-{}", self.tab_counter);
        self.tab_counter += 1;
        let id = tab.id.clone();
        self.tabs.push(tab);
        self.active_tab_id = Some(id.clone());
        id
    }

    pub fn close_tab(&mut self, id: &str) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(pos);
            if self.active_tab_id.as_deref() == Some(id) {
                self.active_tab_id = self.tabs.last().map(|t| t.id.clone());
            }
        }
    }

    pub fn activate_tab(&mut self, id: &str) {
        self.active_tab_id = Some(id.to_string());
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab_id
            .as_ref()
            .and_then(|id| self.tabs.iter().find(|t| t.id == *id))
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let id = self.active_tab_id.clone()?;
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn navigate(&mut self, url: &str, title: &str) {
        if let Some(tab) = self.active_tab_mut() {
            tab.url = url.to_string();
            tab.title = title.to_string();
            tab.loading = false;
        }
        self.add_history(url, title);
    }

    pub fn add_history(&mut self, url: &str, title: &str) {
        if let Some(entry) = self.history_map.get_mut(url) {
            entry.visit_count += 1;
            entry.timestamp = chrono::Utc::now().timestamp();
        } else {
            let entry = HistoryEntry::new(url, title);
            self.history_map.insert(url.to_string(), entry.clone());
            self.history.push(entry);
        }
    }

    pub fn add_bookmark(&mut self, url: &str, title: &str, folder: &str) -> String {
        let bm = Bookmark::new(url, title, folder);
        let id = bm.id.clone();
        self.bookmarks.push(bm);
        id
    }

    pub fn remove_bookmark(&mut self, id: &str) {
        self.bookmarks.retain(|b| b.id != id);
    }

    pub fn get_bookmarks(&self, folder: &str) -> Vec<&Bookmark> {
        self.bookmarks
            .iter()
            .filter(|b| b.folder == folder)
            .collect()
    }

    pub fn get_folders(&self) -> &[String] {
        &self.bookmark_folders
    }

    pub fn search_history(&self, query: &str) -> Vec<&HistoryEntry> {
        let q = query.to_lowercase();
        self.history
            .iter()
            .filter(|h| h.title.to_lowercase().contains(&q) || h.url.to_lowercase().contains(&q))
            .collect()
    }

    pub fn recent_history(&self, limit: usize) -> Vec<&HistoryEntry> {
        let mut sorted = self.history.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted.truncate(limit);
        sorted
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
}

impl Default for BrowserManager {
    fn default() -> Self {
        Self::new()
    }
}
