//! Sleek, monochrome Homepage for False browser.
//! Zero dynamic background pads or canvas fog: flush, clean, pitch-black aesthetic.

use eframe::egui::{self, Color32, CornerRadius, Frame, Key, Margin, Response, RichText, Sense, Ui, Vec2};
use crate::theme::Theme;

pub struct WelcomeAction {
    pub navigate_to: Option<String>,
    pub hovered_link: Option<String>,
}

impl Default for WelcomeAction {
    fn default() -> Self {
        Self {
            navigate_to: None,
            hovered_link: None,
        }
    }
}

pub fn render_welcome_page(
    ui: &mut Ui,
    search_input: &mut String,
) -> WelcomeAction {
    let mut action = WelcomeAction::default();

    ui.add_space(60.0);

    ui.vertical_centered(|ui| {
        // --- 1. Minimalist Wordmark: FALSE ---
        ui.label(
            RichText::new("FALSE")
                .size(76.0)
                .strong()
                .color(Theme::FG)
        );

        // Underline Accent Pill (matching Familiar .ul)
        let (ul_rect, _) = ui.allocate_exact_size(Vec2::new(32.0, 2.5), Sense::hover());
        ui.painter().rect_filled(ul_rect, CornerRadius::same(1), Theme::ACCENT);

        ui.add_space(14.0);

        ui.label(
            RichText::new("FAST • ZERO BLOAT • PURE RUST")
                .size(11.5)
                .color(Theme::MUTE)
        );

        ui.add_space(36.0);

        // --- 2. Central Search Box (matching Familiar .search) ---
        let search_box_width = 620.0f32.min(ui.available_width() - 40.0);

        Frame::new()
            .fill(Theme::B3)
            .stroke(Theme::border_stroke())
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.set_width(search_box_width);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);

                    // Search Magnifying Glass Icon
                    ui.label(RichText::new("🔍").size(15.0).color(Theme::MUTE));

                    // Text Input Field
                    let text_edit = egui::TextEdit::singleline(search_input)
                        .hint_text("Search the web or enter address...")
                        .desired_width(f32::INFINITY)
                        .text_color(Theme::FG)
                        .font(egui::TextStyle::Body);

                    let response: Response = ui.add(text_edit);

                    if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        let query = search_input.trim().to_string();
                        if !query.is_empty() {
                            let target = if query.starts_with("false://") || query.contains("://") {
                                query
                            } else if query.contains('.') && !query.contains(' ') {
                                format!("https://{}", query)
                            } else {
                                format!("https://duckduckgo.com/html/?q={}", query)
                            };
                            action.navigate_to = Some(target);
                        }
                    }

                    // "Enter ↵" Shortcut Badge
                    Frame::new()
                        .fill(Color32::from_rgba_premultiplied(255, 255, 255, 12))
                        .stroke(Theme::border_stroke())
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(6, 2))
                        .show(ui, |ui| {
                            ui.label(RichText::new("↵ Enter").size(10.0).color(Theme::DIM));
                        });
                });
            });

        ui.add_space(28.0);

        // --- 3. Quick Launch Bookmarks Strip ---
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

            let quick_sites = [
                ("🧪 Acid1 CSS Test", "https://www.w3.org/Style/CSS/Test/CSS1/current/test5526c.htm"),
                ("🦀 Rust Language", "https://www.rust-lang.org"),
                ("📖 Wikipedia", "https://en.wikipedia.org"),
                ("⚡ Hacker News", "https://news.ycombinator.com"),
                ("⚙ Settings", "false://settings"),
                ("📜 History", "false://history"),
                ("★ Bookmarks", "false://bookmarks"),
            ];

            for (label, url) in quick_sites {
                let btn = ui.add(
                    egui::Button::new(
                        RichText::new(label)
                            .size(12.0)
                            .color(Theme::FG)
                    )
                    .corner_radius(CornerRadius::same(8))
                    .fill(Theme::B2)
                    .stroke(Theme::border_stroke())
                    .min_size(Vec2::new(0.0, 32.0))
                );

                if btn.hovered() {
                    action.hovered_link = Some(url.to_string());
                }

                if btn.clicked() {
                    action.navigate_to = Some(url.to_string());
                }
            }
        });

        ui.add_space(44.0);

        // --- 4. Pure Rust Architecture Overview Card ---
        Frame::new()
            .fill(Theme::B2)
            .stroke(Theme::border_stroke())
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(20))
            .show(ui, |ui| {
                ui.set_max_width(search_box_width);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("FALSE CORE ENGINE PIPELINE")
                            .size(11.0)
                            .strong()
                            .color(Theme::MUTE)
                    );
                    ui.add_space(10.0);

                    ui.horizontal_wrapped(|ui| {
                        let stages = [
                            ("1. false-net", "HTTP/1.1 & TLS 1.3"),
                            ("2. false-dom", "WHATWG Tokenizer"),
                            ("3. false-css", "CSS3 Rules & Cascade"),
                            ("4. false-layout", "Box Model & Flow"),
                            ("5. false-render", "2D Display List Blitter"),
                        ];

                        for (step, desc) in stages {
                            Frame::new()
                                .fill(Theme::B1)
                                .stroke(Theme::border_stroke())
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::symmetric(12, 10))
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(step).strong().size(12.0).color(Theme::FG));
                                        ui.label(RichText::new(desc).size(10.0).color(Theme::MUTE));
                                    });
                                });
                            ui.add_space(4.0);
                        }
                    });
                });
            });
    });

    ui.add_space(60.0);

    action
}
