use crate::dom::NodeType;
use crate::style::{Display, StyledNode};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

impl Dimensions {
    pub fn padding_box(&self) -> Rect {
        self.content.expanded_by(self.padding)
    }

    pub fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }

    pub fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

impl Rect {
    pub fn expanded_by(&self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

#[derive(Debug, Clone)]
pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>,
}

impl<'a> LayoutBox<'a> {
    pub fn new(box_type: BoxType<'a>) -> LayoutBox<'a> {
        LayoutBox {
            dimensions: Default::default(),
            box_type,
            children: Vec::new(),
        }
    }

    pub fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block has no styled node"),
        }
    }

    pub fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => self.layout_inline(containing_block),
            BoxType::AnonymousBlock => self.layout_block(containing_block),
        }
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        // 1. Calculate width
        self.calculate_block_width(containing_block);

        // 2. Calculate position
        self.calculate_block_position(containing_block);

        // 3. Layout children
        self.layout_block_children();

        // 4. Calculate height
        self.calculate_block_height();
    }

    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = match self.box_type {
            BoxType::BlockNode(node) => &node.computed,
            _ => return,
        };

        let auto_width = style.width.is_none();
        let total_padding = style.padding_left + style.padding_right;
        let total_border = style.border_width * 2.0;

        self.dimensions.padding.left = style.padding_left;
        self.dimensions.padding.right = style.padding_right;
        self.dimensions.padding.top = style.padding_top;
        self.dimensions.padding.bottom = style.padding_bottom;

        self.dimensions.border.left = style.border_width;
        self.dimensions.border.right = style.border_width;
        self.dimensions.border.top = style.border_width;
        self.dimensions.border.bottom = style.border_width;

        self.dimensions.margin.left = style.margin_left;
        self.dimensions.margin.right = style.margin_right;
        self.dimensions.margin.top = style.margin_top;
        self.dimensions.margin.bottom = style.margin_bottom;

        if auto_width {
            let underflow = containing_block.content.width
                - (self.dimensions.margin.left
                    + self.dimensions.margin.right
                    + total_padding
                    + total_border);
            self.dimensions.content.width = underflow.max(0.0);
        } else {
            self.dimensions.content.width = style.width.unwrap_or(containing_block.content.width);
        }
    }

    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        self.dimensions.content.x = containing_block.content.x
            + self.dimensions.margin.left
            + self.dimensions.border.left
            + self.dimensions.padding.left;

        self.dimensions.content.y = containing_block.content.height
            + containing_block.content.y
            + self.dimensions.margin.top
            + self.dimensions.border.top
            + self.dimensions.padding.top;
    }

    fn layout_block_children(&mut self) {
        let mut d = self.dimensions;
        for child in &mut self.children {
            child.layout(d);
            // Track height increment
            d.content.height += child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        if let BoxType::BlockNode(node) = self.box_type {
            if let Some(h) = node.computed.height {
                self.dimensions.content.height = h;
                return;
            }
        }

        // Height is sum of children's margin boxes, or text height if this is a text leaf
        let mut children_height = 0.0;
        for child in &self.children {
            children_height += child.dimensions.margin_box().height;
        }

        if children_height == 0.0 {
            // Check if text element
            if let BoxType::BlockNode(node) = self.box_type {
                if let NodeType::Text(t) = &node.node.node_type {
                    if !t.trim().is_empty() {
                        children_height = (node.computed.font_size * node.computed.line_height).max(18.0);
                    }
                }
            }
        }

        self.dimensions.content.height = children_height;
    }

    fn layout_inline(&mut self, containing_block: Dimensions) {
        let style = match self.box_type {
            BoxType::InlineNode(node) => &node.computed,
            _ => return,
        };

        self.dimensions.padding.left = style.padding_left;
        self.dimensions.padding.right = style.padding_right;
        self.dimensions.padding.top = style.padding_top;
        self.dimensions.padding.bottom = style.padding_bottom;

        self.dimensions.border.left = style.border_width;
        self.dimensions.border.right = style.border_width;
        self.dimensions.border.top = style.border_width;
        self.dimensions.border.bottom = style.border_width;

        self.dimensions.margin.left = style.margin_left;
        self.dimensions.margin.right = style.margin_right;

        self.dimensions.content.x = containing_block.content.x
            + self.dimensions.margin.left
            + self.dimensions.border.left
            + self.dimensions.padding.left;
        self.dimensions.content.y = containing_block.content.y
            + containing_block.content.height
            + self.dimensions.border.top
            + self.dimensions.padding.top;

        // Measure text or inline dimensions
        let mut estimated_width = 0.0;
        let estimated_height = (style.font_size * style.line_height).max(16.0);

        if let BoxType::InlineNode(node) = self.box_type {
            if let NodeType::Text(t) = &node.node.node_type {
                let char_count = t.trim().chars().count();
                // Estimate character width based on font-size
                let char_width = style.font_size * 0.6;
                estimated_width = (char_count as f32 * char_width).min(containing_block.content.width);
            }
        }

        self.dimensions.content.width = style.width.unwrap_or(estimated_width);
        self.dimensions.content.height = style.height.unwrap_or(estimated_height);

        // Layout any inline children
        for child in &mut self.children {
            child.layout(self.dimensions);
        }
    }
}

pub fn build_layout_tree<'a>(root: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let mut root_box = match root.display() {
        Display::Block => LayoutBox::new(BoxType::BlockNode(root)),
        Display::Inline => LayoutBox::new(BoxType::InlineNode(root)),
        Display::None => panic!("Root node cannot have display: none"),
    };

    for child in &root.children {
        match child.display() {
            Display::Block => root_box.children.push(build_layout_tree(child)),
            Display::Inline => root_box.children.push(build_layout_tree(child)),
            Display::None => {} // Skip display: none
        }
    }

    root_box
}
