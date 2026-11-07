//! DOM and HTML parsing for False browser.

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Document,
    Element {
        tag_name: String,
        attributes: Vec<(String, String)>,
    },
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<usize>,
}
