use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
    Comment(String),
    Document,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub id: Option<String>,
    pub classes: HashSet<String>,
}

impl ElementData {
    pub fn new(tag_name: String, attributes: HashMap<String, String>) -> Self {
        let id = attributes.get("id").cloned();
        let classes = attributes
            .get("class")
            .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        Self {
            tag_name: tag_name.to_lowercase(),
            attributes,
            id,
            classes,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Node>,
}

impl Node {
    pub fn text(data: String) -> Self {
        Self {
            node_type: NodeType::Text(data),
            children: Vec::new(),
        }
    }

    pub fn element(name: String, attrs: HashMap<String, String>, children: Vec<Node>) -> Self {
        Self {
            node_type: NodeType::Element(ElementData::new(name, attrs)),
            children,
        }
    }

    pub fn document(children: Vec<Node>) -> Self {
        Self {
            node_type: NodeType::Document,
            children,
        }
    }

    pub fn comment(data: String) -> Self {
        Self {
            node_type: NodeType::Comment(data),
            children: Vec::new(),
        }
    }

    pub fn tag_name(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Element(data) => Some(&data.tag_name),
            _ => None,
        }
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        match &self.node_type {
            NodeType::Element(data) => data.attributes.get(name).map(|s| s.as_str()),
            _ => None,
        }
    }

    pub fn has_class(&self, class_name: &str) -> bool {
        match &self.node_type {
            NodeType::Element(data) => data.classes.contains(class_name),
            _ => false,
        }
    }

    pub fn id(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Element(data) => data.id.as_deref(),
            _ => None,
        }
    }

    pub fn inner_text(&self) -> String {
        let mut text = String::new();
        self.collect_text(&mut text);
        text
    }

    fn collect_text(&self, out: &mut String) {
        match &self.node_type {
            NodeType::Text(t) => out.push_str(t),
            _ => {
                for child in &self.children {
                    child.collect_text(out);
                }
            }
        }
    }

    pub fn pretty_print(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        let mut out = String::new();
        match &self.node_type {
            NodeType::Document => {
                out.push_str(&format!("{}[Document]\n", pad));
            }
            NodeType::Element(el) => {
                let id_s = el.id.as_ref().map(|i| format!("#{}", i)).unwrap_or_default();
                let class_s = if el.classes.is_empty() {
                    String::new()
                } else {
                    format!(".{}", el.classes.iter().cloned().collect::<Vec<_>>().join("."))
                };
                out.push_str(&format!("{}<{}{}{}>\n", pad, el.tag_name, id_s, class_s));
            }
            NodeType::Text(t) => {
                let clean = t.trim();
                if !clean.is_empty() {
                    out.push_str(&format!("{}\"{}\"\n", pad, clean));
                }
            }
            NodeType::Comment(c) => {
                out.push_str(&format!("{}<!-- {} -->\n", pad, c));
            }
        }
        for child in &self.children {
            out.push_str(&child.pretty_print(indent + 1));
        }
        out
    }
}
