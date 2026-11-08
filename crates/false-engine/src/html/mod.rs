use crate::dom::Node;
use std::collections::HashMap;

pub struct HtmlParser {
    pos: usize,
    input: String,
    pub embedded_styles: Vec<String>,
}

impl HtmlParser {
    pub fn new(input: String) -> Self {
        Self {
            pos: 0,
            input,
            embedded_styles: Vec::new(),
        }
    }

    pub fn parse(input: &str) -> (Node, Vec<String>) {
        let mut parser = HtmlParser::new(input.to_string());
        let nodes = parser.parse_nodes();
        let root = if nodes.len() == 1 {
            nodes.into_iter().next().unwrap()
        } else {
            Node::document(nodes)
        };
        (root, parser.embedded_styles)
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn next_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((cur.len_utf8(), ' '));
        self.pos += next_pos;
        cur
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.peek().unwrap()) {
            result.push(self.next_char());
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            }
        }
        nodes
    }

    fn parse_node(&mut self) -> Option<Node> {
        if self.starts_with("<!--") {
            self.parse_comment()
        } else if self.starts_with("<!") {
            self.parse_doctype();
            None
        } else if self.starts_with("<") {
            self.parse_element()
        } else {
            Some(self.parse_text())
        }
    }

    fn parse_comment(&mut self) -> Option<Node> {
        self.pos += 4; // consume <!--
        let mut comment_text = String::new();
        while !self.eof() && !self.starts_with("-->") {
            comment_text.push(self.next_char());
        }
        if self.starts_with("-->") {
            self.pos += 3;
        }
        Some(Node::comment(comment_text.trim().to_string()))
    }

    fn parse_doctype(&mut self) {
        while !self.eof() && self.peek().unwrap() != '>' {
            self.next_char();
        }
        if !self.eof() {
            self.next_char(); // consume '>'
        }
    }

    fn parse_text(&mut self) -> Node {
        let mut text = String::new();
        while !self.eof() && self.peek().unwrap() != '<' {
            text.push(self.next_char());
        }
        let decoded = decode_html_entities(&text);
        Node::text(decoded)
    }

    fn parse_element(&mut self) -> Option<Node> {
        assert_eq!(self.next_char(), '<');
        let tag_name = self
            .consume_while(|c| c.is_ascii_alphanumeric() || c == '-')
            .to_lowercase();
        if tag_name.is_empty() {
            return None;
        }

        let attributes = self.parse_attributes();

        let mut self_closing = false;
        self.consume_whitespace();
        if self.starts_with("/>") {
            self.pos += 2;
            self_closing = true;
        } else if self.starts_with(">") {
            self.pos += 1;
        }

        let is_void = is_void_element(&tag_name) || self_closing;

        // If this is a <style> tag, parse its inner text as CSS directly
        if tag_name == "style" {
            let mut style_css = String::new();
            while !self.eof() && !self.starts_with("</style") {
                style_css.push(self.next_char());
            }
            if self.starts_with("</style") {
                while !self.eof() && self.peek().unwrap() != '>' {
                    self.next_char();
                }
                if !self.eof() {
                    self.next_char();
                }
            }
            self.embedded_styles.push(style_css.clone());
            return Some(Node::element(
                tag_name,
                attributes,
                vec![Node::text(style_css)],
            ));
        }

        let children = if is_void {
            Vec::new()
        } else {
            let ch = self.parse_nodes();
            // Consume closing tag </tag_name>
            if self.starts_with("</") {
                self.pos += 2;
                self.consume_while(|c| c != '>');
                if !self.eof() {
                    self.next_char(); // consume '>'
                }
            }
            ch
        };

        Some(Node::element(tag_name, attributes, children))
    }

    fn parse_attributes(&mut self) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with(">") || self.starts_with("/>") {
                break;
            }
            let name = self.consume_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if name.is_empty() {
                break;
            }
            self.consume_whitespace();
            let value = if self.starts_with("=") {
                self.pos += 1;
                self.consume_whitespace();
                self.parse_attr_value()
            } else {
                name.clone() // Boolean attribute e.g. disabled, checked
            };
            attributes.insert(name.to_lowercase(), value);
        }
        attributes
    }

    fn parse_attr_value(&mut self) -> String {
        let quote = self.peek().unwrap_or(' ');
        if quote == '"' || quote == '\'' {
            self.pos += 1; // consume quote
            let val = self.consume_while(|c| c != quote);
            if !self.eof() {
                self.pos += 1; // consume closing quote
            }
            decode_html_entities(&val)
        } else {
            let val = self.consume_while(|c| !c.is_whitespace() && c != '>' && c != '/');
            decode_html_entities(&val)
        }
    }
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}
