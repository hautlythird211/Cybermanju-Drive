use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub enum NodeType {
    Element(String),
    Text(String),
    Comment,
    Document,
}

#[derive(Debug, Clone)]
pub struct HtmlNode {
    pub node_type: NodeType,
    pub children: Vec<HtmlNode>,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct HtmlDocument {
    pub root: HtmlNode,
    pub raw: String,
}

impl HtmlDocument {
    pub fn parse(html: &str) -> Self {
        let document = Html::parse_document(html);
        let root = Self::scraper_to_node(&document.root_element());
        Self {
            root,
            raw: html.to_string(),
        }
    }

    fn scraper_to_node(element: &scraper::ElementRef) -> HtmlNode {
        let tag = element.value().name().to_string();
        let attrs: Vec<(String, String)> = element
            .value()
            .attrs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let mut children = Vec::new();
        for child in element.children() {
            match child.value() {
                scraper::node::Node::Text(text) => {
                    let txt = text.text.trim();
                    if !txt.is_empty() {
                        children.push(HtmlNode {
                            node_type: NodeType::Text(txt.to_string()),
                            children: vec![],
                            attributes: vec![],
                        });
                    }
                }
                scraper::node::Node::Element(_) => {
                    if let Some(child_elem) = scraper::ElementRef::wrap(child) {
                        children.push(Self::scraper_to_node(&child_elem));
                    }
                }
                _ => {}
            }
        }

        HtmlNode {
            node_type: NodeType::Element(tag),
            children,
            attributes: attrs,
        }
    }

    pub fn extract_text(&self) -> String {
        Self::extract_text_from_node(&self.root)
    }

    fn extract_text_from_node(node: &HtmlNode) -> String {
        match &node.node_type {
            NodeType::Text(t) => t.clone(),
            NodeType::Element(tag) if tag == "script" || tag == "style" => String::new(),
            _ => {
                let mut text = String::new();
                for child in &node.children {
                    text.push_str(&Self::extract_text_from_node(child));
                    text.push(' ');
                }
                text
            }
        }
    }

    pub fn extract_links(&self, base_url: &str) -> Vec<(String, String)> {
        let mut links = Vec::new();
        Self::extract_links_from_node(&self.root, base_url, &mut links);
        links
    }

    fn extract_links_from_node(node: &HtmlNode, base_url: &str, links: &mut Vec<(String, String)>) {
        if let NodeType::Element(tag) = &node.node_type {
            if tag == "a" {
                let href = node
                    .attributes
                    .iter()
                    .find(|(k, _)| k == "href")
                    .map(|(_, v)| v.clone());
                let text = Self::extract_text_from_node(node);
                if let Some(url) = href {
                    if !url.is_empty() && !url.starts_with('#') && !url.starts_with("javascript:") {
                        let full_url = if url.starts_with("http://") || url.starts_with("https://")
                        {
                            url
                        } else if url.starts_with("//") {
                            format!("https:{}", url)
                        } else if url.starts_with('/') {
                            let base = base_url.trim_end_matches('/');
                            format!("{}{}", base, url)
                        } else {
                            let base = base_url.trim_end_matches('/');
                            format!("{}/{}", base, url)
                        };
                        links.push((full_url, text.trim().to_string()));
                    }
                }
            }
        }
        for child in &node.children {
            Self::extract_links_from_node(child, base_url, links);
        }
    }

    pub fn title(&self) -> Option<String> {
        let title_sel = Selector::parse("title").ok()?;
        let doc = Html::parse_document(&self.raw);
        doc.select(&title_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
    }

    pub fn meta_description(&self) -> Option<String> {
        let desc_sel = Selector::parse(r#"meta[name="description"]"#).ok()?;
        let doc = Html::parse_document(&self.raw);
        doc.select(&desc_sel)
            .next()
            .and_then(|e| e.attr("content"))
            .map(|s| s.to_string())
    }
}

impl HtmlNode {
    pub fn is_element(&self, tag: &str) -> bool {
        matches!(&self.node_type, NodeType::Element(t) if t == tag)
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}
