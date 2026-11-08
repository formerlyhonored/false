use crate::css::{Color, CssParser, Rule, SimpleSelector, Specificity, Stylesheet, Value};
use crate::dom::{Node, NodeType};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: Color,
    pub background_color: Color,
    pub font_size: f32,
    pub line_height: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub border_radius: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Inline,
            color: Color::WHITE,
            background_color: Color::TRANSPARENT,
            font_size: 16.0,
            line_height: 1.4,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            border_radius: 0.0,
            width: None,
            height: None,
        }
    }
}

pub type PropertyMap = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub computed: ComputedStyle,
    pub children: Vec<StyledNode<'a>>,
}

impl<'a> StyledNode<'a> {
    pub fn display(&self) -> Display {
        self.computed.display
    }

    pub fn value(&self, name: &str) -> Option<&Value> {
        self.specified_values.get(name)
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or(default))
            .clone()
    }
}

pub fn build_style_tree<'a>(root: &'a Node, stylesheet: &Stylesheet) -> StyledNode<'a> {
    let mut specified_values = match &root.node_type {
        NodeType::Element(_) => specified_values(root, stylesheet),
        _ => HashMap::new(),
    };

    // Parse inline style attribute: e.g. style="color: red; background: blue;"
    if let Some(style_attr) = root.get_attribute("style") {
        let dummy = format!("* {{ {} }}", style_attr);
        let mut p = CssParser::new(&dummy);
        let inline_sheet = p.parse_stylesheet();
        for rule in inline_sheet.rules {
            for decl in rule.declarations {
                specified_values.insert(decl.name, decl.value);
            }
        }
    }

    let computed = compute_styles(root, &specified_values);

    StyledNode {
        node: root,
        specified_values,
        computed,
        children: root
            .children
            .iter()
            .map(|child| build_style_tree(child, stylesheet))
            .collect(),
    }
}

fn compute_styles(node: &Node, specified: &PropertyMap) -> ComputedStyle {
    let mut style = ComputedStyle::default();

    // Default display depending on element tag
    if let Some(tag) = node.tag_name() {
        match tag {
            "html" | "body" | "div" | "header" | "footer" | "main" | "section" | "article"
            | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul" | "ol" | "li" | "form"
            | "nav" => {
                style.display = Display::Block;
            }
            "head" | "script" | "style" | "meta" | "title" | "link" => {
                style.display = Display::None;
            }
            _ => {
                style.display = Display::Inline;
            }
        }
    }

    if let Some(Value::Keyword(d)) = specified.get("display") {
        match d.as_str() {
            "block" => style.display = Display::Block,
            "none" => style.display = Display::None,
            "inline" => style.display = Display::Inline,
            _ => {}
        }
    }

    if let Some(Value::ColorValue(c)) = specified.get("color") {
        style.color = *c;
    }
    if let Some(Value::ColorValue(c)) = specified
        .get("background-color")
        .or_else(|| specified.get("background"))
    {
        style.background_color = *c;
    }

    if let Some(v) = specified.get("font-size") {
        style.font_size = v.to_px();
    }

    // Margins
    if let Some(v) = specified.get("margin") {
        let px = v.to_px();
        style.margin_top = px;
        style.margin_right = px;
        style.margin_bottom = px;
        style.margin_left = px;
    }
    if let Some(v) = specified.get("margin-top") {
        style.margin_top = v.to_px();
    }
    if let Some(v) = specified.get("margin-right") {
        style.margin_right = v.to_px();
    }
    if let Some(v) = specified.get("margin-bottom") {
        style.margin_bottom = v.to_px();
    }
    if let Some(v) = specified.get("margin-left") {
        style.margin_left = v.to_px();
    }

    // Padding
    if let Some(v) = specified.get("padding") {
        let px = v.to_px();
        style.padding_top = px;
        style.padding_right = px;
        style.padding_bottom = px;
        style.padding_left = px;
    }
    if let Some(v) = specified.get("padding-top") {
        style.padding_top = v.to_px();
    }
    if let Some(v) = specified.get("padding-right") {
        style.padding_right = v.to_px();
    }
    if let Some(v) = specified.get("padding-bottom") {
        style.padding_bottom = v.to_px();
    }
    if let Some(v) = specified.get("padding-left") {
        style.padding_left = v.to_px();
    }

    // Borders
    if let Some(v) = specified.get("border-width") {
        style.border_width = v.to_px();
    }
    if let Some(Value::ColorValue(c)) = specified.get("border-color") {
        style.border_color = *c;
    }
    if let Some(v) = specified.get("border-radius") {
        style.border_radius = v.to_px();
    }

    // Dimensions
    if let Some(v) = specified.get("width") {
        style.width = Some(v.to_px());
    }
    if let Some(v) = specified.get("height") {
        style.height = Some(v.to_px());
    }

    style
}

type MatchedRule<'a> = (Specificity, &'a Rule);

fn specified_values(node: &Node, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(node, stylesheet);

    // Sort by specificity
    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}

fn matching_rules<'a>(node: &Node, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(node, rule))
        .collect()
}

fn match_rule<'a>(node: &Node, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches_simple_selector(node, selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matches_simple_selector(node: &Node, selector: &SimpleSelector) -> bool {
    if let Some(ref name) = selector.tag_name {
        if node.tag_name() != Some(name) {
            return false;
        }
    }

    if let Some(ref id) = selector.id {
        if node.id() != Some(id) {
            return false;
        }
    }

    for class in &selector.classes {
        if !node.has_class(class) {
            return false;
        }
    }

    true
}
