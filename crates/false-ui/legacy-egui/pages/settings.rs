//! False Browser Settings Internal Webapp (`false://settings`).

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    Tabs,
    Search,
    Privacy,
    Shortcuts,
}

impl Default for SettingsSection {
    fn default() -> Self {
        Self::Appearance
    }
}

pub struct SettingsConfig {
    pub active_section: SettingsSection,
    pub theme_name: String,
    pub layout_mode: String, // "modern" (sidebar) or "classic" (top tabs)
    pub bookmarks_bar: bool,
    pub quick_links: bool,
    pub sleep_tabs: bool,
    pub sleep_idle_min: u32,
    pub default_search: String,
    pub tracker_blocking: bool,
    pub https_only: bool,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            active_section: SettingsSection::Appearance,
            theme_name: "Nocturne (Pitch Black)".into(),
            layout_mode: "Modern (Sidebar)".into(),
            bookmarks_bar: true,
            quick_links: true,
            sleep_tabs: true,
            sleep_idle_min: 15,
            default_search: "DuckDuckGo".into(),
            tracker_blocking: true,
            https_only: true,
        }
    }
}

pub fn render_settings_page(ui: &mut Ui, cfg: &mut SettingsConfig) {
    ui.columns(2, |columns| {
        // --- Left Navigation Rail (matching Familiar .pnav) ---
        columns[0].set_max_width(220.0);
        columns[0].vertical(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("SETTINGS").size(14.0).strong().color(Theme::FG));
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let nav_items = [
                (SettingsSection::Appearance, "🎨 Appearance"),
                (SettingsSection::Tabs, "📑 Tabs Management"),
                (SettingsSection::Search, "🔍 Search Engine"),
                (SettingsSection::Privacy, "🛡 Privacy & Security"),
                (SettingsSection::Shortcuts, "⌨ Keyboard Shortcuts"),
            ];

            for (sec, label) in nav_items {
                let is_sel = cfg.active_section == sec;
                let bg_color = if is_sel { Theme::B3 } else { Color32::TRANSPARENT };
                let text_color = if is_sel { Theme::FG } else { Theme::MUTE };

                let btn = ui.add(
                    egui::Button::new(RichText::new(label).size(13.0).color(text_color).strong())
                        .fill(bg_color)
                        .corner_radius(CornerRadius::same(6))
                        .stroke(if is_sel { Stroke::new(1.0, Theme::LINE_BRIGHT) } else { Stroke::NONE })
                        .min_size(Vec2::new(200.0, 32.0))
                );

                if btn.clicked() {
                    cfg.active_section = sec;
                }
                ui.add_space(3.0);
            }
        });

        // --- Right Content Body (matching Familiar .pbody) ---
        columns[1].vertical(|ui| {
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .id_salt("settings_content_scroll")
                .show(ui, |ui| {
                    match cfg.active_section {
                        SettingsSection::Appearance => render_appearance_section(ui, cfg),
                        SettingsSection::Tabs => render_tabs_section(ui, cfg),
                        SettingsSection::Search => render_search_section(ui, cfg),
                        SettingsSection::Privacy => render_privacy_section(ui, cfg),
                        SettingsSection::Shortcuts => render_shortcuts_section(ui),
                    }
                });
        });
    });
}

fn render_appearance_section(ui: &mut Ui, cfg: &mut SettingsConfig) {
    ui.label(RichText::new("Appearance & Theme").size(24.0).strong().color(Theme::FG));
    ui.label(RichText::new("Customize the browser chrome, colors, and layout.").size(12.0).color(Theme::MUTE));
    ui.add_space(16.0);

    // Color Theme Card
    Frame::new()
        .fill(Theme::B1)
        .stroke(Theme::border_stroke())
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.label(RichText::new("Color Palette").size(14.0).strong().color(Theme::FG));
            ui.label(RichText::new("Pure monochrome dark mode. Web content styling remains intact.").size(11.5).color(Theme::MUTE));
            ui.add_space(10.0);

            let themes = [
                ("Nocturne (Pitch Black)", "#000000"),
                ("Obsidian (Charcoal)", "#0b0b0b"),
                ("Slate (Steel Grey)", "#14161b"),
                ("Monokai (Warm Dark)", "#181816"),
            ];

            for (name, hex) in themes {
                let is_active = cfg.theme_name == name;
                ui.horizontal(|ui| {
                    if ui.radio(is_active, format!("{} — {}", name, hex)).clicked() {
                        cfg.theme_name = name.into();
                    }
                });
            }
        });

    ui.add_space(14.0);

    // Layout Mode Card
    Frame::new()
        .fill(Theme::B1)
        .stroke(Theme::border_stroke())
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.label(RichText::new("Shell Layout").size(14.0).strong().color(Theme::FG));
            ui.label(RichText::new("Choose between the Familiar vertical sidebar or classic horizontal top tabs.").size(11.5).color(Theme::MUTE));
            ui.add_space(10.0);

            ui.radio_value(&mut cfg.layout_mode, "Modern (Sidebar)".into(), "Modern Sidebar (Left 232px vertical tabs)");
            ui.radio_value(&mut cfg.layout_mode, "Classic (Top Tabs)".into(), "Classic Strip (Horizontal tabs along top bar)");

            ui.add_space(10.0);
            ui.checkbox(&mut cfg.bookmarks_bar, "Show Bookmarks Bar under navigation bar (Ctrl+Shift+B)");
            ui.checkbox(&mut cfg.quick_links, "Show Quick Link shortcuts on New Tab page");
        });
}

fn render_tabs_section(ui: &mut Ui, cfg: &mut SettingsConfig) {
    ui.label(RichText::new("Tab Management").size(24.0).strong().color(Theme::FG));
    ui.label(RichText::new("Optimize memory and tab lifecycle.").size(12.0).color(Theme::MUTE));
    ui.add_space(16.0);

    Frame::new()
        .fill(Theme::B1)
        .stroke(Theme::border_stroke())
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.checkbox(&mut cfg.sleep_tabs, "Sleep background tabs (Unload tabs after inactivity to save RAM)");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Sleep inactive tabs after:").color(Theme::MUTE));
                egui::ComboBox::from_id_salt("sleep_idle_combo")
                    .selected_text(format!("{} minutes", cfg.sleep_idle_min))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cfg.sleep_idle_min, 5, "5 minutes");
                        ui.selectable_value(&mut cfg.sleep_idle_min, 15, "15 minutes");
                        ui.selectable_value(&mut cfg.sleep_idle_min, 30, "30 minutes");
                        ui.selectable_value(&mut cfg.sleep_idle_min, 60, "60 minutes");
                    });
            });
        });
}

fn render_search_section(ui: &mut Ui, cfg: &mut SettingsConfig) {
    ui.label(RichText::new("Default Search Engine").size(24.0).strong().color(Theme::FG));
    ui.label(RichText::new("Used when queries are entered into the Omnibox or Welcome search box.").size(12.0).color(Theme::MUTE));
    ui.add_space(16.0);

    Frame::new()
        .fill(Theme::B1)
        .stroke(Theme::border_stroke())
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            let engines = ["DuckDuckGo", "Google", "Bing", "Brave Search", "Ecosia"];
            for engine in engines {
                ui.radio_value(&mut cfg.default_search, engine.into(), engine);
            }
        });
}

fn render_privacy_section(ui: &mut Ui, cfg: &mut SettingsConfig) {
    ui.label(RichText::new("Privacy & Security").size(24.0).strong().color(Theme::FG));
    ui.label(RichText::new("Built-in protections with zero telemetry.").size(12.0).color(Theme::MUTE));
    ui.add_space(16.0);

    Frame::new()
        .fill(Theme::B1)
        .stroke(Theme::border_stroke())
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.checkbox(&mut cfg.tracker_blocking, "Built-in Tracker & Ad Blocking (Block intrusive analytics and ads)");
            ui.checkbox(&mut cfg.https_only, "HTTPS-Only Mode (Automatically upgrade HTTP connections)");
            ui.add_space(12.0);

            if ui.button(RichText::new("Clear Browsing Cache & History").color(Theme::WARN_AMBER)).clicked() {
                // Trigger clear cache
            }
        });
}

fn render_shortcuts_section(ui: &mut Ui) {
    ui.label(RichText::new("Keyboard Shortcuts").size(24.0).strong().color(Theme::FG));
    ui.label(RichText::new("Productivity keybindings built into False.").size(12.0).color(Theme::MUTE));
    ui.add_space(16.0);

    Frame::new()
        .fill(Theme::B1)
        .stroke(Theme::border_stroke())
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            let shortcuts = [
                ("New Tab", "Ctrl + T"),
                ("Close Active Tab", "Ctrl + W"),
                ("Switch Tab", "Ctrl + Tab"),
                ("Focus Address Bar / Omnibox", "Ctrl + L"),
                ("Toggle Tab Sidebar", "Ctrl + B"),
                ("Toggle Bookmarks Bar", "Ctrl + Shift + B"),
                ("Open History", "Ctrl + H"),
                ("Open Settings", "Ctrl + ,"),
                ("Toggle Developer Tools", "F12"),
                ("Reload Page", "Ctrl + R / F5"),
                ("Zoom In / Out", "Ctrl + + / Ctrl + -"),
                ("Reset Zoom", "Ctrl + 0"),
            ];

            egui::Grid::new("shortcuts_grid")
                .striped(true)
                .min_col_width(200.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Action").strong().color(Theme::MUTE));
                    ui.label(RichText::new("Shortcut").strong().color(Theme::MUTE));
                    ui.end_row();

                    for (action, key) in shortcuts {
                        ui.label(RichText::new(action).color(Theme::FG));
                        ui.monospace(RichText::new(key).color(Theme::ACCENT));
                        ui.end_row();
                    }
                });
        });
}
