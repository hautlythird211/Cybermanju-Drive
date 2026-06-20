mod html;
mod layout;

pub use html::{HtmlDocument, HtmlNode, NodeType};
pub use layout::{LayoutBox, LayoutEngine, LayoutSize};

pub struct HtmlRenderer {
    layout: LayoutEngine,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        Self {
            layout: LayoutEngine::new(),
        }
    }

    pub fn render(&self, html: &str, width: u32) -> HtmlDocument {
        let doc = HtmlDocument::parse(html);
        doc
    }

    pub fn layout(&self, doc: &HtmlDocument, width: u32) -> Vec<LayoutBox> {
        self.layout.layout(doc, width)
    }

    pub fn extract_text(&self, html: &str) -> String {
        let doc = HtmlDocument::parse(html);
        doc.extract_text()
    }

    pub fn extract_links(&self, html: &str, base_url: &str) -> Vec<(String, String)> {
        let doc = HtmlDocument::parse(html);
        doc.extract_links(base_url)
    }

    pub fn extract_title(&self, html: &str) -> Option<String> {
        let doc = HtmlDocument::parse(html);
        doc.title()
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}
