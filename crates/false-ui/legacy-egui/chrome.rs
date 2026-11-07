//! Browser Chrome: Top navigation bar, Omnibox, and Status bar matching Familiar's monochrome design.

use eframe::egui::{self, Color32, CornerRadius, Key, Margin, Response, RichText, Stroke, Ui, Vec2};
use crate::tab::Tab;
use crate::theme::Theme;

pub struct ChromeAction {
    pub navigate_to: Option<String>,
    pub toggle_sidebar: bool,
    pub toggle_devtools: bool,
    pub toggle_bookmarks_bar: bool,
    pub reload: bool,
    pub star_clicked: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub zoom_reset: bool,
}

impl Default for ChromeAction {
    fn default() -> Self {
        Self {
            navigate_to: None,
            toggle_sidebar: false,
            toggle_devtools: false,
            toggle_bookmarks_bar: false,
            reload: false,
            star_clicked: false,
            zoom_in: false,
            zoom_out: false,
            zoom_reset: false,
        }
    }
}

pub fn render_top_bar(
    ui: &mut Ui,
    current_tab: &mut Tab,
    omnibox_input: &mut String,
    sidebar_open: bool,
    devtools_open: bool,
    bookmarks_bar_open: bool,
    is_starred: bool,
) -> ChromeAction {
    let mut action = ChromeAction::default();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

        // --- Left Controls (matching Familiar .l) ---
        // 1. Sidebar Toggle Button
        let side_color = if sidebar_open { Theme::FG } else { Theme::MUTE };
        let side_btn = ui.add(
            egui::Button::new(RichText::new("◫").size(16.0).color(side_color))
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(32.0, 32.0))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
        ).on_hover_text("Toggle Tab Sidebar (Ctrl+B)");

        if side_btn.clicked() {
            action.toggle_sidebar = true;
        }

        // 2. Back Button
        let back_color = if current_tab.can_go_back() { Theme::MUTE } else { Theme::DIM };
        let back_btn = ui.add_enabled(
            current_tab.can_go_back(),
            egui::Button::new(RichText::new("←").size(16.0).color(back_color))
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(32.0, 32.0))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
        );
        if back_btn.clicked() {
            current_tab.go_back();
            *omnibox_input = current_tab.url.clone();
        }

        // 3. Forward Button
        let fwd_color = if current_tab.can_go_forward() { Theme::MUTE } else { Theme::DIM };
        let fwd_btn = ui.add_enabled(
            current_tab.can_go_forward(),
            egui::Button::new(RichText::new("→").size(16.0).color(fwd_color))
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(32.0, 32.0))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
        );
        if fwd_btn.clicked() {
            current_tab.go_forward();
            *omnibox_input = current_tab.url.clone();
        }

        // 4. Reload Button
        let reload_sym = if current_tab.is_loading { "✕" } else { "⟳" };
        let reload_btn = ui.add(
            egui::Button::new(RichText::new(reload_sym).size(16.0).color(Theme::MUTE))
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(32.0, 32.0))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
        );
        if reload_btn.clicked() {
            action.reload = true;
            action.navigate_to = Some(current_tab.url.clone());
        }

        // 5. Home Button
        let home_btn = ui.add(
            egui::Button::new(RichText::new("⌂").size(16.0).color(Theme::MUTE))
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(32.0, 32.0))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
        );
        if home_btn.clicked() {
            action.navigate_to = Some("false://welcome".into());
            *omnibox_input = "false://welcome".into();
        }

        ui.add_space(4.0);

        // --- Center: Address Bar (matching Familiar .addr) ---
        let available_width = ui.available_width() - 210.0;
        let addr_width = available_width.clamp(140.0, 680.0);

        let _addr_frame = egui::Frame::new()
            .fill(Theme::B3)
            .stroke(Theme::border_stroke())
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(10, 5))
            .show(ui, |ui| {
                ui.set_width(addr_width);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                    // Security / Protocol Icon
                    let (sec_icon, sec_color, sec_tooltip) = if omnibox_input.starts_with("https://") {
                        ("🔒", Theme::LOCK_GREEN, "Connection encrypted via TLS 1.3")
                    } else if omnibox_input.starts_with("false://") {
                        ("⚡", Theme::ACCENT, "Internal False Page")
                    } else if omnibox_input.starts_with("file://") {
                        ("📁", Theme::MUTE, "Local Document")
                    } else {
                        ("⚠️", Theme::WARN_AMBER, "Plaintext HTTP")
                    };

                    ui.label(RichText::new(sec_icon).color(sec_color).size(13.0))
                        .on_hover_text(sec_tooltip);

                    // Text Input
                    let text_edit = egui::TextEdit::singleline(omnibox_input)
                        .hint_text("Enter website address or false://settings...")
                        .desired_width(f32::INFINITY)
                        .text_color(Theme::FG)
                        .font(egui::TextStyle::Body);

                    let response: Response = ui.add(text_edit);

                    if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        let mut target = omnibox_input.trim().to_string();
                        if !target.contains("://") && !target.is_empty() {
                            if target.contains('.') {
                                target = format!("https://{}", target);
                            } else {
                                target = format!("https://duckduckgo.com/html/?q={}", target);
                            }
                        }
                        *omnibox_input = target.clone();
                        action.navigate_to = Some(target);
                    }

                    // Bookmark Star
                    let star_color = if is_starred {
                        Theme::STAR_YELLOW
                    } else {
                        Theme::DIM
                    };

                    let star_btn = ui.add(
                        egui::Button::new(RichText::new("★").size(14.0).color(star_color))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                    ).on_hover_text("Bookmark this page");

                    if star_btn.clicked() {
                        action.star_clicked = true;
                    }
                });
            });

        ui.add_space(6.0);

        // --- Right Controls (matching Familiar .r) ---
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 1. Settings Shortcut Button
            let set_btn = ui.add(
                egui::Button::new(RichText::new("⚙").size(15.0).color(Theme::MUTE))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(Vec2::new(32.0, 32.0))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
            ).on_hover_text("Open False Settings (Ctrl+,)");

            if set_btn.clicked() {
                action.navigate_to = Some("false://settings".into());
                *omnibox_input = "false://settings".into();
            }

            // 2. History Shortcut Button
            let hist_btn = ui.add(
                egui::Button::new(RichText::new("📜").size(15.0).color(Theme::MUTE))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(Vec2::new(32.0, 32.0))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
            ).on_hover_text("Open Browsing History (Ctrl+H)");

            if hist_btn.clicked() {
                action.navigate_to = Some("false://history".into());
                *omnibox_input = "false://history".into();
            }

            // 3. DevTools Toggle Button
            let devtools_color = if devtools_open { Theme::FG } else { Theme::MUTE };
            let devtools_bg = if devtools_open { Theme::B4 } else { Color32::TRANSPARENT };
            let devtools_btn = ui.add(
                egui::Button::new(RichText::new("🛠").size(15.0).color(devtools_color))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(Vec2::new(32.0, 32.0))
                    .fill(devtools_bg)
                    .stroke(Stroke::NONE)
            ).on_hover_text("Toggle Developer Tools Drawer [F12]");

            if devtools_btn.clicked() {
                action.toggle_devtools = true;
            }

            // 4. Bookmarks Bar Toggle Button
            let bbar_color = if bookmarks_bar_open { Theme::FG } else { Theme::MUTE };
            let bbar_bg = if bookmarks_bar_open { Theme::B4 } else { Color32::TRANSPARENT };
            let bbar_btn = ui.add(
                egui::Button::new(RichText::new("★").size(14.0).color(bbar_color))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(Vec2::new(32.0, 32.0))
                    .fill(bbar_bg)
                    .stroke(Stroke::NONE)
            ).on_hover_text("Toggle Bookmarks Bar (Ctrl+Shift+B)");

            if bbar_btn.clicked() {
                action.toggle_bookmarks_bar = true;
            }

            // 5. Zoom Indicator
            let zoom_btn = ui.add(
                egui::Button::new(
                    RichText::new(format!("{:.0}%", current_tab.zoom * 100.0))
                        .size(11.0)
                        .color(Theme::MUTE)
                        .strong()
                )
                .corner_radius(CornerRadius::same(6))
                .fill(Theme::B2)
                .stroke(Theme::border_stroke())
            ).on_hover_text("Reset Zoom (Click) or use Ctrl+/Ctrl-");

            if zoom_btn.clicked() {
                action.zoom_reset = true;
            }
        });
    });

    action
}

pub fn render_bookmarks_bar(
    ui: &mut Ui,
    bookmarks: &[(&str, &str)],
    omnibox_input: &mut String,
) -> Option<String> {
    let mut target_navigation = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

        for (name, url) in bookmarks {
            let btn = ui.add(
                egui::Button::new(
                    RichText::new(*name)
                        .size(11.5)
                        .color(Theme::MUTE)
                )
                .corner_radius(CornerRadius::same(6))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
            ).on_hover_text(*url);

            if btn.clicked() {
                target_navigation = Some((*url).to_string());
                *omnibox_input = (*url).to_string();
            }
        }
    });

    target_navigation
}

pub fn render_status_bar(
    ui: &mut Ui,
    status_text: &str,
    _zoom: f32,
    viewport_size: Vec2,
) -> ChromeAction {
    let mut action = ChromeAction::default();

    ui.horizontal(|ui| {
        // Status text on left
        ui.label(
            RichText::new(status_text)
                .size(11.5)
                .color(Theme::MUTE)
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Engine Spec
            ui.label(
                RichText::new("FALSE • PURE RUST • ZERO WEBVIEWS")
                    .size(10.5)
                    .color(Theme::DIM)
            );

            ui.separator();

            // Dimensions
            ui.label(
                RichText::new(format!("{:.0} × {:.0} px", viewport_size.x, viewport_size.y))
                    .size(10.5)
                    .color(Theme::DIM)
            );

            ui.separator();

            // Zoom +/- buttons
            if ui.button(RichText::new("+").size(11.0).color(Theme::MUTE)).clicked() {
                action.zoom_in = true;
            }
            if ui.button(RichText::new("-").size(11.0).color(Theme::MUTE)).clicked() {
                action.zoom_out = true;
            }
        });
    });

    action
}
