use super::html::{HtmlDocument, HtmlNode, NodeType};

#[derive(Debug, Clone)]
pub struct LayoutSize {
    pub width: f32,
    pub height: f32,
}

impl LayoutSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone)]
pub enum LayoutType {
    Block,
    Inline,
    Text,
}

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub layout_type: LayoutType,
    pub tag: String,
    pub text: Option<String>,
    pub children: Vec<LayoutBox>,
    pub size: LayoutSize,
    pub position: (f32, f32),
    pub computed_style: StyleProps,
}

#[derive(Debug, Clone)]
pub struct StyleProps {
    pub color: String,
    pub background: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub display: String,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub text_align: String,
}

impl Default for StyleProps {
    fn default() -> Self {
        Self {
            color: "#00ff41".to_string(),
            background: "transparent".to_string(),
            font_size: 14.0,
            font_weight: 400,
            display: "block".to_string(),
            margin_top: 0.0,
            margin_bottom: 8.0,
            margin_left: 0.0,
            margin_right: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            padding_right: 0.0,
            text_align: "left".to_string(),
        }
    }
}

pub struct LayoutEngine {
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self { }
    }

    pub fn layout(&self, doc: &HtmlDocument, width: u32) -> Vec<LayoutBox> {
        let mut boxes = Vec::new();
        let (mut x, mut y) = (0.0, 0.0);

        for child in &doc.root.children {
            let layout_box = self.build_layout(child, width as f32, &mut x, &mut y);
            boxes.push(layout_box);
        }

        boxes
    }

    fn build_layout(&self, node: &HtmlNode, max_width: f32, x: &mut f32, y: &mut f32) -> LayoutBox {
        match &node.node_type {
            NodeType::Text(text) => {
                let box_w = (text.len() as f32 * 8.0).min(max_width);
                let box_h = 18.0;
                let pos = (*x, *y);
                *y += box_h;

                LayoutBox {
                    layout_type: LayoutType::Text,
                    tag: "text".to_string(),
                    text: Some(text.clone()),
                    children: vec![],
                    size: LayoutSize::new(box_w, box_h),
                    position: pos,
                    computed_style: self.style_for_tag("text"),
                }
            }
            NodeType::Element(tag) => {
                let tag_style = self.style_for_tag(tag);
                let mut children = Vec::new();
                let mut cx = *x + tag_style.margin_left + tag_style.padding_left;
                let mut cy = *y + tag_style.margin_top + tag_style.padding_top;
                let initial_cy = cy;

                for child in &node.children {
                    let child_box = self.build_layout(child, max_width - cx, &mut cx, &mut cy);
                    cx += child_box.size.width;
                    children.push(child_box);
                }

                let total_h = cy - initial_cy + tag_style.padding_bottom + tag_style.margin_bottom;
                let total_w = max_width;

                let pos = (*x, *y);
                *y += total_h;

                LayoutBox {
                    layout_type: LayoutType::Block,
                    tag: tag.clone(),
                    text: None,
                    children,
                    size: LayoutSize::new(total_w, total_h),
                    position: pos,
                    computed_style: tag_style,
                }
            }
            _ => LayoutBox {
                layout_type: LayoutType::Block,
                tag: "unknown".to_string(),
                text: None,
                children: vec![],
                size: LayoutSize::new(0.0, 0.0),
                position: (0.0, 0.0),
                computed_style: StyleProps::default(),
            },
        }
    }

    fn style_for_tag(&self, tag: &str) -> StyleProps {
        let mut style = StyleProps::default();
        match tag {
            "h1" => {
                style.font_size = 28.0;
                style.font_weight = 700;
                style.margin_bottom = 16.0;
                style.color = "#5af0ff".to_string();
            }
            "h2" => {
                style.font_size = 22.0;
                style.font_weight = 600;
                style.margin_bottom = 12.0;
                style.color = "#5af0ff".to_string();
            }
            "h3" => {
                style.font_size = 18.0;
                style.font_weight = 600;
                style.margin_bottom = 10.0;
            }
            "a" => {
                style.color = "#b388ff".to_string();
                style.display = "inline".to_string();
                style.margin_bottom = 0.0;
            }
            "img" => {
                style.display = "inline".to_string();
                style.margin_bottom = 0.0;
            }
            "p" => {
                style.margin_bottom = 12.0;
            }
            "div" => {}
            "span" => {
                style.display = "inline".to_string();
                style.margin_bottom = 0.0;
            }
            "pre" => {
                style.font_size = 12.0;
                style.background = "rgba(0,255,65,0.05)".to_string();
            }
            "code" => {
                style.font_size = 12.0;
                style.background = "rgba(0,255,65,0.05)".to_string();
            }
            "blockquote" => {
                style.margin_left = 16.0;
                style.color = "#3a86ff".to_string();
            }
            "li" => {
                style.margin_bottom = 4.0;
            }
            "ul" | "ol" => {
                style.margin_left = 24.0;
            }
            _ => {}
        }
        style
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}
