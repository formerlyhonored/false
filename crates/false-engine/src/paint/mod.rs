use crate::css::Color;
use crate::dom::NodeType;
use crate::layout::{BoxType, LayoutBox, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayCommand {
    SolidColor(Color, Rect),
    Border(Color, Rect, f32),
    Text(String, Rect, Color, f32),
}

pub type DisplayList = Vec<DisplayCommand>;

pub fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);
    render_text(list, layout_box);

    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    let style = match layout_box.box_type {
        BoxType::BlockNode(node) | BoxType::InlineNode(node) => &node.computed,
        BoxType::AnonymousBlock => return,
    };

    if style.background_color.a > 0 {
        list.push(DisplayCommand::SolidColor(
            style.background_color,
            layout_box.dimensions.border_box(),
        ));
    }
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    let style = match layout_box.box_type {
        BoxType::BlockNode(node) | BoxType::InlineNode(node) => &node.computed,
        BoxType::AnonymousBlock => return,
    };

    if style.border_width > 0.0 && style.border_color.a > 0 {
        list.push(DisplayCommand::Border(
            style.border_color,
            layout_box.dimensions.border_box(),
            style.border_width,
        ));
    }
}

fn render_text(list: &mut DisplayList, layout_box: &LayoutBox) {
    let node = match layout_box.box_type {
        BoxType::BlockNode(snode) | BoxType::InlineNode(snode) => snode,
        BoxType::AnonymousBlock => return,
    };

    if let NodeType::Text(text) = &node.node.node_type {
        let clean = text.trim();
        if !clean.is_empty() {
            list.push(DisplayCommand::Text(
                clean.to_string(),
                layout_box.dimensions.content,
                node.computed.color,
                node.computed.font_size,
            ));
        }
    }
}

// ============================================================================
// Software Rasterizer Canvas
// ============================================================================

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA (4 bytes per pixel)
}

impl Canvas {
    pub fn new(width: u32, height: u32, bg: Color) -> Self {
        let total_bytes = (width * height * 4) as usize;
        let mut pixels = Vec::with_capacity(total_bytes);
        for _ in 0..(width * height) {
            pixels.push(bg.r);
            pixels.push(bg.g);
            pixels.push(bg.b);
            pixels.push(bg.a);
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn rasterize(&mut self, display_list: &[DisplayCommand]) {
        for item in display_list {
            match item {
                DisplayCommand::SolidColor(color, rect) => {
                    self.fill_rect(rect, *color);
                }
                DisplayCommand::Border(color, rect, width) => {
                    self.stroke_rect(rect, *width, *color);
                }
                DisplayCommand::Text(text, rect, color, font_size) => {
                    self.draw_text(text, rect.x as i32, rect.y as i32, *font_size, *color);
                }
            }
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        if color.a == 255 {
            self.pixels[idx] = color.r;
            self.pixels[idx + 1] = color.g;
            self.pixels[idx + 2] = color.b;
            self.pixels[idx + 3] = 255;
        } else if color.a > 0 {
            // Alpha blending
            let alpha = color.a as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;
            self.pixels[idx] = ((color.r as f32 * alpha) + (self.pixels[idx] as f32 * inv_alpha)) as u8;
            self.pixels[idx + 1] = ((color.g as f32 * alpha) + (self.pixels[idx + 1] as f32 * inv_alpha)) as u8;
            self.pixels[idx + 2] = ((color.b as f32 * alpha) + (self.pixels[idx + 2] as f32 * inv_alpha)) as u8;
            self.pixels[idx + 3] = 255;
        }
    }

    pub fn fill_rect(&mut self, rect: &Rect, color: Color) {
        let x0 = (rect.x.floor() as i32).clamp(0, self.width as i32);
        let y0 = (rect.y.floor() as i32).clamp(0, self.height as i32);
        let x1 = ((rect.x + rect.width).ceil() as i32).clamp(0, self.width as i32);
        let y1 = ((rect.y + rect.height).ceil() as i32).clamp(0, self.height as i32);

        for y in y0..y1 {
            for x in x0..x1 {
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn stroke_rect(&mut self, rect: &Rect, stroke_width: f32, color: Color) {
        let w = stroke_width.ceil() as i32;
        let x0 = rect.x.floor() as i32;
        let y0 = rect.y.floor() as i32;
        let x1 = (rect.x + rect.width).ceil() as i32;
        let y1 = (rect.y + rect.height).ceil() as i32;

        // Top border
        for y in y0..(y0 + w) {
            for x in x0..x1 { self.set_pixel(x, y, color); }
        }
        // Bottom border
        for y in (y1 - w)..y1 {
            for x in x0..x1 { self.set_pixel(x, y, color); }
        }
        // Left border
        for y in y0..y1 {
            for x in x0..(x0 + w) { self.set_pixel(x, y, color); }
        }
        // Right border
        for y in y0..y1 {
            for x in (x1 - w)..x1 { self.set_pixel(x, y, color); }
        }
    }

    pub fn draw_text(&mut self, text: &str, start_x: i32, start_y: i32, font_size: f32, color: Color) {
        let scale = (font_size / 8.0).max(1.0).round() as i32;
        let char_spacing = 8 * scale;
        let mut cur_x = start_x;
        let cur_y = start_y;

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = start_x;
                continue;
            }
            self.draw_char(ch, cur_x, cur_y, scale, color);
            cur_x += char_spacing;
            if cur_x >= self.width as i32 {
                break;
            }
        }
    }

    fn draw_char(&mut self, ch: char, x0: i32, y0: i32, scale: i32, color: Color) {
        let glyph = get_glyph_bitmap(ch);
        for row in 0..8 {
            let byte = glyph[row];
            for col in 0..8 {
                if (byte & (1 << (7 - col))) != 0 {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            self.set_pixel(x0 + (col as i32) * scale + dx, y0 + (row as i32) * scale + dy, color);
                        }
                    }
                }
            }
        }
    }

    /// Exports pixels as standard 32-bit uncompressed Windows BMP
    pub fn to_bmp(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let file_header_size = 14;
        let info_header_size = 40;
        let pixel_data_size = (self.width * self.height * 4) as usize;
        let file_size = file_header_size + info_header_size + pixel_data_size;

        // BITMAPFILEHEADER
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&(file_size as u32).to_le_bytes());
        out.extend_from_slice(&[0, 0, 0, 0]); // reserved
        out.extend_from_slice(&((file_header_size + info_header_size) as u32).to_le_bytes()); // offset

        // BITMAPINFOHEADER
        out.extend_from_slice(&(info_header_size as u32).to_le_bytes());
        out.extend_from_slice(&(self.width as i32).to_le_bytes());
        out.extend_from_slice(&(-(self.height as i32)).to_le_bytes()); // negative for top-down bitmap
        out.extend_from_slice(&(1u16).to_le_bytes()); // planes
        out.extend_from_slice(&(32u16).to_le_bytes()); // bpp (32 bit BGRA)
        out.extend_from_slice(&[0, 0, 0, 0]); // compression (BI_RGB)
        out.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
        out.extend_from_slice(&[0; 16]); // ppm resolution & colors

        // Pixel data (Windows 32-bit BMP expects BGRA)
        for chunk in self.pixels.chunks_exact(4) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            let a = chunk[3];
            out.push(b);
            out.push(g);
            out.push(r);
            out.push(a);
        }

        out
    }

    /// Encodes BMP as a Base64 data URL string ready for <img> or webview display
    pub fn to_base64_bmp_data_url(&self) -> String {
        let bmp_bytes = self.to_bmp();
        format!("data:image/bmp;base64,{}", base64_encode(&bmp_bytes))
    }
}

fn base64_encode(input: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        out.push(CHARSET[(b0 >> 2) as usize] as char);
        out.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// Clean, built-in 8x8 font glyphs for ASCII characters
fn get_glyph_bitmap(ch: char) -> [u8; 8] {
    match ch {
        'A' => [0x18, 0x24, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x00],
        'B' => [0x7C, 0x42, 0x42, 0x7C, 0x42, 0x42, 0x7C, 0x00],
        'C' => [0x3C, 0x42, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00],
        'D' => [0x78, 0x44, 0x42, 0x42, 0x42, 0x44, 0x78, 0x00],
        'E' => [0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x7E, 0x00],
        'F' => [0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x40, 0x00],
        'G' => [0x3C, 0x42, 0x40, 0x4E, 0x42, 0x42, 0x3C, 0x00],
        'H' => [0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
        'I' => [0x3E, 0x1C, 0x18, 0x18, 0x18, 0x1C, 0x3E, 0x00],
        'J' => [0x1F, 0x04, 0x04, 0x04, 0x44, 0x44, 0x38, 0x00],
        'K' => [0x42, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x00],
        'L' => [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00],
        'M' => [0x63, 0x77, 0x5B, 0x49, 0x49, 0x41, 0x41, 0x00],
        'N' => [0x42, 0x62, 0x52, 0x4A, 0x46, 0x42, 0x42, 0x00],
        'O' => [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'P' => [0x7C, 0x42, 0x42, 0x7C, 0x40, 0x40, 0x40, 0x00],
        'Q' => [0x3C, 0x42, 0x42, 0x42, 0x4A, 0x44, 0x3A, 0x00],
        'R' => [0x7C, 0x42, 0x42, 0x7C, 0x48, 0x44, 0x42, 0x00],
        'S' => [0x3C, 0x42, 0x40, 0x3C, 0x02, 0x42, 0x3C, 0x00],
        'T' => [0x7F, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x00],
        'U' => [0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'V' => [0x42, 0x42, 0x42, 0x42, 0x24, 0x24, 0x18, 0x00],
        'W' => [0x41, 0x49, 0x49, 0x55, 0x55, 0x22, 0x22, 0x00],
        'X' => [0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x42, 0x00],
        'Y' => [0x41, 0x22, 0x14, 0x08, 0x08, 0x08, 0x08, 0x00],
        'Z' => [0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x7E, 0x00],
        'a' => [0x00, 0x00, 0x3C, 0x02, 0x3E, 0x42, 0x3B, 0x00],
        'b' => [0x40, 0x40, 0x5C, 0x62, 0x42, 0x62, 0x5C, 0x00],
        'c' => [0x00, 0x00, 0x3C, 0x42, 0x40, 0x42, 0x3C, 0x00],
        'd' => [0x02, 0x02, 0x3A, 0x46, 0x42, 0x46, 0x3A, 0x00],
        'e' => [0x00, 0x00, 0x3C, 0x42, 0x7E, 0x40, 0x3C, 0x00],
        'f' => [0x0C, 0x12, 0x10, 0x3C, 0x10, 0x10, 0x10, 0x00],
        'g' => [0x00, 0x00, 0x3A, 0x46, 0x46, 0x3E, 0x02, 0x3C],
        'h' => [0x40, 0x40, 0x5C, 0x62, 0x42, 0x42, 0x42, 0x00],
        'i' => [0x10, 0x00, 0x30, 0x10, 0x10, 0x10, 0x38, 0x00],
        'j' => [0x04, 0x00, 0x0C, 0x04, 0x04, 0x24, 0x18, 0x00],
        'k' => [0x40, 0x40, 0x48, 0x50, 0x60, 0x50, 0x48, 0x00],
        'l' => [0x30, 0x10, 0x10, 0x10, 0x10, 0x10, 0x38, 0x00],
        'm' => [0x00, 0x00, 0x66, 0x5B, 0x49, 0x49, 0x49, 0x00],
        'n' => [0x00, 0x00, 0x5C, 0x62, 0x42, 0x42, 0x42, 0x00],
        'o' => [0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'p' => [0x00, 0x00, 0x5C, 0x62, 0x62, 0x5C, 0x40, 0x40],
        'q' => [0x00, 0x00, 0x3A, 0x46, 0x46, 0x3A, 0x02, 0x02],
        'r' => [0x00, 0x00, 0x5A, 0x64, 0x40, 0x40, 0x40, 0x00],
        's' => [0x00, 0x00, 0x3E, 0x40, 0x3C, 0x02, 0x7C, 0x00],
        't' => [0x10, 0x10, 0x3C, 0x10, 0x10, 0x12, 0x0C, 0x00],
        'u' => [0x00, 0x00, 0x42, 0x42, 0x42, 0x46, 0x3A, 0x00],
        'v' => [0x00, 0x00, 0x42, 0x42, 0x24, 0x24, 0x18, 0x00],
        'w' => [0x00, 0x00, 0x41, 0x49, 0x49, 0x55, 0x22, 0x00],
        'x' => [0x00, 0x00, 0x42, 0x24, 0x18, 0x24, 0x42, 0x00],
        'y' => [0x00, 0x00, 0x42, 0x42, 0x46, 0x3A, 0x02, 0x3C],
        'z' => [0x00, 0x00, 0x7E, 0x04, 0x08, 0x10, 0x7E, 0x00],
        '0' => [0x3C, 0x42, 0x46, 0x4A, 0x52, 0x62, 0x3C, 0x00],
        '1' => [0x18, 0x28, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00],
        '2' => [0x3C, 0x42, 0x02, 0x0C, 0x30, 0x40, 0x7E, 0x00],
        '3' => [0x3C, 0x42, 0x02, 0x1C, 0x02, 0x42, 0x3C, 0x00],
        '4' => [0x04, 0x0C, 0x14, 0x24, 0x7E, 0x04, 0x04, 0x00],
        '5' => [0x7E, 0x40, 0x7C, 0x02, 0x02, 0x42, 0x3C, 0x00],
        '6' => [0x1C, 0x20, 0x40, 0x7C, 0x42, 0x42, 0x3C, 0x00],
        '7' => [0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x20, 0x00],
        '8' => [0x3C, 0x42, 0x42, 0x3C, 0x42, 0x42, 0x3C, 0x00],
        '9' => [0x3C, 0x42, 0x42, 0x3E, 0x02, 0x04, 0x38, 0x00],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x10],
        '!' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00],
        '?' => [0x3C, 0x42, 0x02, 0x0C, 0x18, 0x00, 0x18, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '+' => [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00],
        ':' => [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00],
        '/' => [0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x00],
        '(' => [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00],
        ')' => [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00],
        _ =>   [0x00, 0x7E, 0x42, 0x42, 0x42, 0x7E, 0x00, 0x00], // Box fallback
    }
}
