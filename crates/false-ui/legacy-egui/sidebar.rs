//! Vertical Sidebar for False browser, styled directly after Familiar.

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Sense, Stroke, Ui, Vec2};
use crate::tab::Tab;
use crate::theme::Theme;

pub struct SidebarAction {
    pub select_tab: Option<usize>,
    pub close_tab: Option<usize>,
    pub new_tab: bool,
    pub navigate_to: Option<String>,
}

impl Default for SidebarAction {
    fn default() -> Self {
        Self {
            select_tab: None,
            close_tab: None,
            new_tab: false,
            navigate_to: None,
        }
    }
}

pub fn render_sidebar(
    ui: &mut Ui,
    tabs: &[Tab],
    active_tab_idx: usize,
) -> SidebarAction {
    let mut action = SidebarAction::default();

    ui.vertical(|ui| {
        // --- 1. Brand Row (matching Familiar .brand) ---
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

            // Brand Wordmark
            ui.label(
                RichText::new("FALSE")
                    .size(15.0)
                    .strong()
                    .color(Theme::FG)
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // New Tab Button (+ icon, 26x26px, 6px radius)
                let new_btn = ui.add(
                    egui::Button::new(
                        RichText::new("+").size(16.0).color(Theme::MUTE).strong()
                    )
                    .corner_radius(CornerRadius::same(6))
                    .min_size(Vec2::new(26.0, 26.0))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                );

                if new_btn.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if new_btn.clicked() {
                    action.new_tab = true;
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // --- 2. Scrollable Tabs Area (matching Familiar .tabs) ---
        let available_height = ui.available_height() - 110.0;
        egui::ScrollArea::vertical()
            .id_salt("sidebar_tabs_scroll")
            .max_height(available_height.max(100.0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 3.0);

                for (idx, tab) in tabs.iter().enumerate() {
                    let is_active = idx == active_tab_idx;

                    let bg_color = if is_active {
                        Theme::B3 // #1d1d1d
                    } else {
                        Color32::TRANSPARENT
                    };

                    let stroke = if is_active {
                        Stroke::new(1.0, Theme::LINE_BRIGHT)
                    } else {
                        Stroke::NONE
                    };

                    let text_color = if is_active {
                        Theme::FG // #f2f2f2
                    } else {
                        Theme::MUTE // #8e8e93
                    };

                    let frame_res = egui::Frame::new()
                        .fill(bg_color)
                        .stroke(stroke)
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            let available_width = ui.available_width();
                            ui.set_width(available_width);

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                                // Active Left Indicator Bar (matching Familiar .tab.on::before)
                                if is_active {
                                    let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(2.5, 18.0), Sense::hover());
                                    ui.painter().rect_filled(
                                        bar_rect,
                                        CornerRadius::same(1),
                                        Theme::ACCENT,
                                    );
                                } else {
                                    ui.allocate_exact_size(Vec2::new(2.5, 18.0), Sense::hover());
                                }

                                // Favicon Box (20x20px, 6px radius)
                                let (fav_rect, _) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::hover());
                                ui.painter().rect_filled(
                                    fav_rect,
                                    CornerRadius::same(6),
                                    Color32::from_rgba_premultiplied(255, 255, 255, 20),
                                );

                                let fav_icon = if let Some(p) = &tab.page {
                                    match p.as_str() {
                                        "settings" => "⚙",
                                        "history" => "📜",
                                        "bookmarks" => "★",
                                        _ => "⚡",
                                    }
                                } else if tab.url.starts_with("https://") {
                                    "🔒"
                                } else if tab.url.starts_with("false://") {
                                    "⚡"
                                } else {
                                    "📄"
                                };

                                ui.painter().text(
                                    fav_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    fav_icon,
                                    egui::FontId::proportional(11.0),
                                    Theme::FG,
                                );

                                // Tab Title (truncated smoothly)
                                let title_text = if tab.title.len() > 17 {
                                    format!("{}...", &tab.title[..15])
                                } else {
                                    tab.title.clone()
                                };

                                let title_label = ui.add(
                                    egui::Label::new(
                                        RichText::new(title_text)
                                            .size(13.0)
                                            .color(text_color)
                                            .strong()
                                    )
                                    .sense(Sense::click())
                                );

                                if title_label.clicked() {
                                    action.select_tab = Some(idx);
                                }

                                // Truncate / align close button to the right
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if tabs.len() > 1 {
                                        let close_btn = ui.add(
                                            egui::Button::new(
                                                RichText::new("×")
                                                    .size(14.0)
                                                    .color(Theme::DIM)
                                            )
                                            .corner_radius(CornerRadius::same(6))
                                            .min_size(Vec2::new(18.0, 18.0))
                                            .fill(Color32::TRANSPARENT)
                                            .stroke(Stroke::NONE)
                                        );

                                        if close_btn.clicked() {
                                            action.close_tab = Some(idx);
                                        }
                                    }
                                });
                            });
                        });

                    if frame_res.response.interact(Sense::click()).clicked() {
                        action.select_tab = Some(idx);
                    }
                }
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // --- 3. Bottom Sidebar Navigation Items (matching Familiar sidebar bottom) ---
        let bottom_links = [
            ("⚙ Settings", "false://settings"),
            ("📜 History", "false://history"),
            ("★ Bookmarks", "false://bookmarks"),
        ];

        for (label, url) in bottom_links {
            let btn = ui.add(
                egui::Button::new(RichText::new(label).size(12.0).color(Theme::MUTE))
                    .corner_radius(CornerRadius::same(6))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                    .min_size(Vec2::new(200.0, 26.0))
            );

            if btn.clicked() {
                action.navigate_to = Some(url.to_string());
            }
        }
    });

    action
}
