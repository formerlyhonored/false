//! Main False Browser Application State & Event Loop.

use eframe::egui::{self, CentralPanel, Key, Margin, Modifiers, Panel, Stroke, Ui};
use crate::tab::Tab;
use crate::chrome::{render_top_bar, render_bookmarks_bar, render_status_bar};
use crate::sidebar::render_sidebar;
use crate::devtools::{DevToolsState, render_devtools_drawer};
use crate::viewport::render_viewport;
use crate::pages::settings::SettingsConfig;
use crate::pages::history::{HistoryState, HistoryRecord};
use crate::pages::bookmarks::{BookmarksState, BookmarkItem};
use crate::theme::Theme;

pub struct FalseBrowserApp {
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
    pub next_tab_id: usize,
    pub omnibox_input: String,
    pub homepage_search_input: String,
    pub sidebar_open: bool,
    pub devtools_open: bool,
    pub bookmarks_bar_open: bool,
    pub is_starred: bool,
    pub settings_cfg: SettingsConfig,
    pub history_state: HistoryState,
    pub bookmarks_state: BookmarksState,
    pub devtools_state: DevToolsState,
    pub devtools_height: f32,
    pub hovered_link: Option<String>,
    pub quick_bookmarks: Vec<(&'static str, &'static str)>,
}

impl Default for FalseBrowserApp {
    fn default() -> Self {
        let default_url = "false://welcome";
        let initial_tab = Tab::new(1, default_url);
        Self {
            tabs: vec![initial_tab],
            active_tab_idx: 0,
            next_tab_id: 2,
            omnibox_input: default_url.to_string(),
            homepage_search_input: String::new(),
            sidebar_open: true,
            devtools_open: false,
            bookmarks_bar_open: false,
            is_starred: false,
            settings_cfg: SettingsConfig::default(),
            history_state: HistoryState::default(),
            bookmarks_state: BookmarksState::default(),
            devtools_state: DevToolsState::default(),
            devtools_height: 250.0,
            hovered_link: None,
            quick_bookmarks: vec![
                ("⚡ Welcome", "false://welcome"),
                ("⚙ Settings", "false://settings"),
                ("📜 History", "false://history"),
                ("🧪 Acid1 Test", "https://www.w3.org/Style/CSS/Test/CSS1/current/test5526c.htm"),
                ("🦀 Rust Lang", "https://www.rust-lang.org"),
                ("📖 Wikipedia", "https://en.wikipedia.org"),
            ],
        }
    }
}

impl FalseBrowserApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_idx]
    }

    fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_idx]
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // F12: Toggle DevTools
        if ctx.input(|i| i.key_pressed(Key::F12)) {
            self.devtools_open = !self.devtools_open;
        }

        // Ctrl + B: Toggle Sidebar
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::B)) {
            self.sidebar_open = !self.sidebar_open;
        }

        // Ctrl + Shift + B: Toggle Bookmarks Bar
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::B)) {
            self.bookmarks_bar_open = !self.bookmarks_bar_open;
        }

        // Ctrl + H: Open History
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::H)) {
            self.open_new_tab("false://history");
        }

        // Ctrl + ,: Open Settings
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Comma)) {
            self.open_new_tab("false://settings");
        }

        // Ctrl + T: New Tab
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::T)) {
            self.open_new_tab("false://welcome");
        }

        // Ctrl + W: Close Active Tab
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::W)) {
            self.close_tab(self.active_tab_idx);
        }

        // Ctrl + R: Reload
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::R)) {
            let url = self.active_tab().url.clone();
            self.active_tab_mut().navigate(url);
        }

        // Ctrl + Tab: Switch Tab
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Tab)) {
            if !self.tabs.is_empty() {
                self.active_tab_idx = (self.active_tab_idx + 1) % self.tabs.len();
                self.omnibox_input = self.active_tab().url.clone();
            }
        }
    }

    pub fn open_new_tab(&mut self, url: impl Into<String>) {
        let tab = Tab::new(self.next_tab_id, url);
        self.next_tab_id += 1;
        self.tabs.push(tab);
        self.active_tab_idx = self.tabs.len() - 1;
        self.omnibox_input = self.active_tab().url.clone();
    }

    pub fn close_tab(&mut self, idx: usize) {
        if self.tabs.len() > 1 && idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.active_tab_idx >= self.tabs.len() {
                self.active_tab_idx = self.tabs.len() - 1;
            }
            self.omnibox_input = self.active_tab().url.clone();
        }
    }

    fn record_history_entry(&mut self, url: &str, title: &str) {
        let host = if let Some(domain) = url.strip_prefix("https://") {
            domain.split('/').next().unwrap_or(domain).to_string()
        } else if let Some(domain) = url.strip_prefix("http://") {
            domain.split('/').next().unwrap_or(domain).to_string()
        } else {
            url.to_string()
        };

        self.history_state.records.insert(0, HistoryRecord {
            id: self.history_state.records.len() + 1,
            title: title.to_string(),
            url: url.to_string(),
            host,
            time_ago: "Just now".into(),
        });

        if self.history_state.records.len() > 300 {
            self.history_state.records.pop();
        }
    }
}

impl eframe::App for FalseBrowserApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_keyboard_shortcuts(&ctx);

        let mut next_navigation = None;
        self.hovered_link = None;

        // Check if current URL is bookmarked
        let current_url = self.active_tab().url.clone();
        self.is_starred = self.bookmarks_state.items.iter().any(|b| b.url == current_url);

        // --- 1. Top Header Navigation Bar (matching Familiar .top, 52px) ---
        Panel::top("top_header_panel")
            .frame(egui::Frame::new()
                .fill(Theme::BG)
                .stroke(Stroke::new(1.0, Theme::LINE))
                .inner_margin(Margin::symmetric(12, 8)))
            .show(ui, |ui| {
                let current_tab = &mut self.tabs[self.active_tab_idx];
                let chrome_action = render_top_bar(
                    ui,
                    current_tab,
                    &mut self.omnibox_input,
                    self.sidebar_open,
                    self.devtools_open,
                    self.bookmarks_bar_open,
                    self.is_starred,
                );

                if let Some(target) = chrome_action.navigate_to {
                    next_navigation = Some(target);
                }
                if chrome_action.toggle_sidebar {
                    self.sidebar_open = !self.sidebar_open;
                }
                if chrome_action.toggle_devtools {
                    self.devtools_open = !self.devtools_open;
                }
                if chrome_action.toggle_bookmarks_bar {
                    self.bookmarks_bar_open = !self.bookmarks_bar_open;
                }
                if chrome_action.star_clicked {
                    let cur_url = current_tab.url.clone();
                    let cur_title = current_tab.title.clone();
                    if let Some(pos) = self.bookmarks_state.items.iter().position(|b| b.url == cur_url) {
                        self.bookmarks_state.items.remove(pos);
                    } else {
                        self.bookmarks_state.items.push(BookmarkItem {
                            title: cur_title,
                            url: cur_url.clone(),
                            host: cur_url,
                        });
                    }
                }
            });

        // Bookmarks Bar (matching Familiar .bbar)
        if self.bookmarks_bar_open {
            Panel::top("bookmarks_bar_panel")
                .frame(egui::Frame::new()
                    .fill(Theme::BG)
                    .stroke(Stroke::new(1.0, Theme::LINE))
                    .inner_margin(Margin::symmetric(12, 4)))
                .show(ui, |ui| {
                    if let Some(target) = render_bookmarks_bar(ui, &self.quick_bookmarks, &mut self.omnibox_input) {
                        next_navigation = Some(target);
                    }
                });
        }

        // --- 2. Bottom Status Bar ---
        let current_zoom = self.active_tab().zoom;
        let viewport_size = ui.available_size();
        let status_message = if let Some(link) = &self.hovered_link {
            format!("Go to {}", link)
        } else {
            "Ready".to_string()
        };

        Panel::bottom("bottom_status_bar")
            .frame(egui::Frame::new()
                .fill(Theme::BG)
                .stroke(Stroke::new(1.0, Theme::LINE))
                .inner_margin(Margin::symmetric(12, 4)))
            .show(ui, |ui| {
                let status_action = render_status_bar(
                    ui,
                    &status_message,
                    current_zoom,
                    viewport_size,
                );

                if status_action.zoom_in {
                    self.active_tab_mut().zoom = (self.active_tab().zoom + 0.1).min(3.0);
                }
                if status_action.zoom_out {
                    self.active_tab_mut().zoom = (self.active_tab().zoom - 0.1).max(0.3);
                }
                if status_action.zoom_reset {
                    self.active_tab_mut().zoom = 1.0;
                }
            });

        // --- 3. Developer Tools Drawer (Bottom Panel if open) ---
        if self.devtools_open {
            Panel::bottom("devtools_drawer")
                .resizable(true)
                .default_size(self.devtools_height)
                .size_range(120.0..=600.0)
                .frame(egui::Frame::new()
                    .fill(Theme::B1)
                    .stroke(Stroke::new(1.0, Theme::LINE))
                    .inner_margin(Margin::same(12)))
                .show(ui, |ui| {
                    let current_tab = &mut self.tabs[self.active_tab_idx];
                    render_devtools_drawer(ui, &mut self.devtools_state, current_tab);
                });
        }

        // --- 4. Left Sidebar (matching Familiar .side, 232px wide) ---
        if self.sidebar_open {
            Panel::left("left_tab_sidebar")
                .resizable(false)
                .default_size(232.0)
                .frame(egui::Frame::new()
                    .fill(Theme::BG)
                    .stroke(Stroke::new(1.0, Theme::LINE))
                    .inner_margin(Margin::symmetric(10, 10)))
                .show(ui, |ui| {
                    let sb_action = render_sidebar(ui, &self.tabs, self.active_tab_idx);

                    if sb_action.new_tab {
                        self.open_new_tab("false://welcome");
                    }
                    if let Some(close_idx) = sb_action.close_tab {
                        self.close_tab(close_idx);
                    }
                    if let Some(sel_idx) = sb_action.select_tab {
                        self.active_tab_idx = sel_idx;
                        self.omnibox_input = self.active_tab().url.clone();
                    }
                    if let Some(target) = sb_action.navigate_to {
                        next_navigation = Some(target);
                    }
                });
        }

        // --- 5. Central Web Viewport (matching Familiar .view, flush, no padded background) ---
        CentralPanel::default()
            .frame(egui::Frame::new().fill(Theme::BG))
            .show(ui, |ui| {
                let current_tab = &mut self.tabs[self.active_tab_idx];
                let vp_action = render_viewport(
                    ui,
                    current_tab,
                    &mut self.homepage_search_input,
                    &mut self.settings_cfg,
                    &mut self.history_state,
                    &mut self.bookmarks_state,
                );

                if let Some(target) = vp_action.navigate_to {
                    next_navigation = Some(target);
                }
                if let Some(link) = vp_action.hovered_link {
                    self.hovered_link = Some(link);
                }
            });

        // Execute deferred navigation if requested
        if let Some(url) = next_navigation {
            self.active_tab_mut().navigate(url.clone());
            self.omnibox_input = url.clone();
            self.homepage_search_input.clear();
            let title = self.active_tab().title.clone();
            self.record_history_entry(&url, &title);
        }
    }
}
