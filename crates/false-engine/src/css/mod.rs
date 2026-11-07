#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        if s == "transparent" {
            return Some(Self::TRANSPARENT);
        }
        if s == "black" { return Some(Self::BLACK); }
        if s == "white" { return Some(Self::WHITE); }
        if s == "red" { return Some(Color::new(235, 75, 75, 255)); }
        if s == "green" { return Some(Color::new(75, 185, 95, 255)); }
        if s == "blue" { return Some(Color::new(75, 135, 245, 255)); }
        if s == "yellow" { return Some(Color::new(245, 215, 65, 255)); }
        if s == "gray" || s == "grey" { return Some(Color::new(140, 145, 155, 255)); }
        if s == "orange" { return Some(Color::new(245, 145, 65, 255)); }
        if s == "purple" { return Some(Color::new(175, 95, 245, 255)); }

        if s.starts_with('#') {
            let hex = &s[1..];
            if hex.len() == 3 {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                return Some(Color::new(r, g, b, 255));
            } else if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::new(r, g, b, 255));
            } else if hex.len() == 8 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                return Some(Color::new(r, g, b, a));
            }
        }

        if s.starts_with("rgb(") && s.ends_with(')') {
            let inner = &s[4..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 3 {
                let r = parts[0].trim().parse().ok()?;
                let g = parts[1].trim().parse().ok()?;
                let b = parts[2].trim().parse().ok()?;
                return Some(Color::new(r, g, b, 255));
            }
        }

        if s.starts_with("rgba(") && s.ends_with(')') {
            let inner = &s[5..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 4 {
                let r = parts[0].trim().parse().ok()?;
                let g = parts[1].trim().parse().ok()?;
                let b = parts[2].trim().parse().ok()?;
                let a_f: f32 = parts[3].trim().parse().ok()?;
                let a = (a_f.clamp(0.0, 1.0) * 255.0) as u8;
                return Some(Color::new(r, g, b, a));
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px,
    Em,
    Rem,
    Percent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    Number(f32),
}

impl Value {
    pub fn to_px(&self) -> f32 {
        match self {
            Value::Length(val, Unit::Px) => *val,
            Value::Length(val, Unit::Em) => *val * 16.0,
            Value::Length(val, Unit::Rem) => *val * 16.0,
            Value::Number(n) => *n,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

pub type Specificity = (usize, usize, usize);

impl SimpleSelector {
    pub fn specificity(&self) -> Specificity {
        let id_count = if self.id.is_some() { 1 } else { 0 };
        let class_count = self.classes.len();
        let tag_count = if self.tag_name.is_some() { 1 } else { 0 };
        (id_count, class_count, tag_count)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selectors: Vec<SimpleSelector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn parse(source: &str) -> Self {
        let mut parser = CssParser::new(source);
        parser.parse_stylesheet()
    }

    pub fn default_user_agent_stylesheet() -> Self {
        let ua_css = r#"
            html, body {
                display: block;
                background-color: #18191f;
                color: #f1f2f6;
                margin: 0px;
                padding: 16px;
                font-size: 16px;
            }
            div {
                display: block;
            }
            h1 {
                display: block;
                font-size: 32px;
                margin: 16px 0px;
                color: #ffffff;
            }
            h2 {
                display: block;
                font-size: 24px;
                margin: 14px 0px;
                color: #ffffff;
            }
            h3 {
                display: block;
                font-size: 18px;
                margin: 12px 0px;
            }
            p {
                display: block;
                margin: 10px 0px;
                line-height: 1.5;
            }
            span {
                display: inline;
            }
            button {
                display: inline;
                background-color: #3b4252;
                color: #ffffff;
                padding: 8px 14px;
                border-radius: 6px;
                border-width: 1px;
                border-color: #4c566a;
            }
        "#;
        Self::parse(ua_css)
    }
}

pub struct CssParser {
    pos: usize,
    input: String,
}

impl CssParser {
    pub fn new(input: &str) -> Self {
        Self {
            pos: 0,
            input: input.to_string(),
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
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
        while !self.eof() {
            let c = self.peek().unwrap();
            if c.is_whitespace() {
                self.next_char();
            } else if self.input[self.pos..].starts_with("/*") {
                // Skip CSS comments
                self.pos += 2;
                while !self.eof() && !self.input[self.pos..].starts_with("*/") {
                    self.next_char();
                }
                if !self.eof() {
                    self.pos += 2;
                }
            } else {
                break;
            }
        }
    }

    pub fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut rules = Vec::new();
        while !self.eof() {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            }
        }
        Stylesheet { rules }
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selectors();
        if selectors.is_empty() {
            return None;
        }
        let declarations = self.parse_declarations();
        Some(Rule {
            selectors,
            declarations,
        })
    }

    fn parse_selectors(&mut self) -> Vec<SimpleSelector> {
        let mut selectors = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.peek() == Some('{') {
                break;
            }
            selectors.push(self.parse_simple_selector());
            self.consume_whitespace();
            if self.peek() == Some(',') {
                self.next_char(); // consume comma
            } else {
                break;
            }
        }
        selectors
    }

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            classes: Vec::new(),
        };

        while !self.eof() {
            match self.peek() {
                Some('#') => {
                    self.next_char();
                    selector.id = Some(self.parse_identifier());
                }
                Some('.') => {
                    self.next_char();
                    selector.classes.push(self.parse_identifier());
                }
                Some('*') => {
                    self.next_char();
                }
                Some(c) if c.is_ascii_alphanumeric() || c == '-' => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }

        selector
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        self.consume_whitespace();
        if self.peek() != Some('{') {
            return Vec::new();
        }
        self.next_char(); // consume '{'

        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.peek() == Some('}') {
                break;
            }
            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            }
            self.consume_whitespace();
            if self.peek() == Some(';') {
                self.next_char();
            }
        }

        if !self.eof() && self.peek() == Some('}') {
            self.next_char(); // consume '}'
        }

        declarations
    }

    fn parse_declaration(&mut self) -> Option<Declaration> {
        let property_name = self.parse_identifier().to_lowercase();
        self.consume_whitespace();
        if self.peek() != Some(':') {
            return None;
        }
        self.next_char(); // consume ':'
        self.consume_whitespace();
        let value = self.parse_value();
        Some(Declaration {
            name: property_name,
            value,
        })
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }

    fn parse_value(&mut self) -> Value {
        let raw = self.consume_while(|c| c != ';' && c != '}').trim().to_string();

        if let Some(color) = Color::parse(&raw) {
            return Value::ColorValue(color);
        }

        if raw.ends_with("px") {
            if let Ok(n) = raw[..raw.len() - 2].trim().parse::<f32>() {
                return Value::Length(n, Unit::Px);
            }
        } else if raw.ends_with("em") {
            if let Ok(n) = raw[..raw.len() - 2].trim().parse::<f32>() {
                return Value::Length(n, Unit::Em);
            }
        } else if raw.ends_with("rem") {
            if let Ok(n) = raw[..raw.len() - 3].trim().parse::<f32>() {
                return Value::Length(n, Unit::Rem);
            }
        } else if raw.ends_with('%') {
            if let Ok(n) = raw[..raw.len() - 1].trim().parse::<f32>() {
                return Value::Length(n, Unit::Percent);
            }
        } else if let Ok(n) = raw.parse::<f32>() {
            return Value::Number(n);
        }

        Value::Keyword(raw.to_lowercase())
    }
}
