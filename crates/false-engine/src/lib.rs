pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod paint;
pub mod style;

use css::{Color, Stylesheet};
use html::HtmlParser;
use layout::{build_layout_tree, Dimensions};
use paint::{build_display_list, Canvas, DisplayList};
use style::build_style_tree;

pub struct RenderOutput {
    pub width: u32,
    pub height: u32,
    pub dom_tree: String,
    pub display_list: DisplayList,
    pub canvas: Canvas,
}

impl RenderOutput {
    pub fn to_bmp(&self) -> Vec<u8> {
        self.canvas.to_bmp()
    }

    pub fn to_data_url(&self) -> String {
        self.canvas.to_base64_bmp_data_url()
    }
}

pub struct Engine;

impl Engine {
    pub fn render(html_input: &str, width: u32, height: u32) -> RenderOutput {
        // 1. Parse HTML & collect embedded <style> tags
        let (root_node, embedded_styles) = HtmlParser::parse(html_input);
        let dom_tree = root_node.pretty_print(0);

        // 2. Parse User-Agent stylesheet + all document <style> sheets
        let mut stylesheet = Stylesheet::default_user_agent_stylesheet();
        for style_str in embedded_styles {
            let parsed_sheet = Stylesheet::parse(&style_str);
            stylesheet.rules.extend(parsed_sheet.rules);
        }

        // 3. Style tree construction (Cascading & Specificity matching)
        let style_root = build_style_tree(&root_node, &stylesheet);

        // 4. Layout calculation (Box model & geometry)
        let mut layout_root = build_layout_tree(&style_root);
        let mut viewport: Dimensions = Default::default();
        viewport.content.width = width as f32;
        viewport.content.height = 0.0;
        layout_root.layout(viewport);

        // 5. Display list generation
        let display_list = build_display_list(&layout_root);

        // 6. Rasterization to pixel buffer
        let bg_color = Color::new(22, 23, 29, 255); // False dark slate floor
        let mut canvas = Canvas::new(width, height, bg_color);
        canvas.rasterize(&display_list);

        RenderOutput {
            width,
            height,
            dom_tree,
            display_list,
            canvas,
        }
    }
}
