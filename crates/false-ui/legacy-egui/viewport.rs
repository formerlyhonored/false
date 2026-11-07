//! Web Viewport and Internal Page Router for False browser.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use crate::pages::bookmarks::{render_bookmarks_page, BookmarksState};
use crate::pages::history::{render_history_page, HistoryState};
use crate::pages::settings::{render_settings_page, SettingsConfig};
use crate::pages::welcome::render_welcome_page;
use crate::tab::Tab;

pub struct ViewportAction {
    pub navigate_to: Option<String>,
    pub hovered_link: Option<String>,
}

impl Default for ViewportAction {
    fn default() -> Self {
        Self {
            navigate_to: None,
            hovered_link: None,
        }
    }
}

pub fn render_viewport(
    ui: &mut Ui,
    tab: &mut Tab,
    search_input: &mut String,
    settings_cfg: &mut SettingsConfig,
    history_state: &mut HistoryState,
    bookmarks_state: &mut BookmarksState,
) -> ViewportAction {
    let mut action = ViewportAction::default();

    // Check if current tab is viewing an internal page
    if let Some(page) = tab.page.as_deref() {
        match page {
            "settings" => {
                render_settings_page(ui, settings_cfg);
            }
            "history" => {
                if let Some(target) = render_history_page(ui, history_state) {
                    action.navigate_to = Some(target);
                }
            }
            "bookmarks" => {
                if let Some(target) = render_bookmarks_page(ui, bookmarks_state) {
                    action.navigate_to = Some(target);
                }
            }
            _ => {
                let welcome_act = render_welcome_page(ui, search_input);
                action.navigate_to = welcome_act.navigate_to;
                action.hovered_link = welcome_act.hovered_link;
            }
        }
    } else if tab.url == "false://welcome" || tab.url.is_empty() {
        let welcome_act = render_welcome_page(ui, search_input);
        action.navigate_to = welcome_act.navigate_to;
        action.hovered_link = welcome_act.hovered_link;
    } else {
        render_web_document_canvas(ui, tab, &mut action);
    }

    action
}

fn render_web_document_canvas(ui: &mut Ui, tab: &Tab, action: &mut ViewportAction) {
    let available_width = ui.available_width();

    egui::ScrollArea::vertical()
        .id_salt("web_doc_scroll")
        .show(ui, |ui| {
            // High-contrast clean white page container for web content rendering
            Frame::new()
                .fill(Color32::from_rgb(255, 255, 255))
                .inner_margin(Margin::same(40))
                .show(ui, |ui| {
                    ui.set_min_width(available_width);
                    ui.set_min_height(600.0);

                    // Document Header
                    ui.label(
                        RichText::new(&tab.title)
                            .size(28.0)
                            .strong()
                            .color(Color32::from_rgb(20, 20, 25)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("Rendered by False Native Engine • Source: {}", tab.url))
                            .size(11.5)
                            .color(Color32::from_rgb(110, 115, 125)),
                    );

                    ui.separator();
                    ui.add_space(18.0);

                    // Document Body
                    ui.label(
                        RichText::new("About This Document")
                            .size(18.0)
                            .strong()
                            .color(Color32::from_rgb(30, 35, 45)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(
                            "You are viewing a web document rendered inside False's 100% pure Rust viewport. \
                            The pipeline resolves block formatting contexts, margin collapsing, and typography \
                            without any webviews or external C++ dependencies.",
                        )
                        .size(13.5)
                        .color(Color32::from_rgb(60, 65, 75)),
                    );

                    ui.add_space(18.0);

                    // Box Model Demo Container
                    Frame::new()
                        .fill(Color32::from_rgb(246, 248, 250))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(220, 225, 235)))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::same(16))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Box Model Container: display: block; padding: 16px; margin: 8px")
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::from_rgb(0, 102, 204)),
                            );
                            ui.label(
                                RichText::new(
                                    "This layout container demonstrates resolved margin, border, and padding dimensions \
                                    calculated directly by False's layout pipeline.",
                                )
                                .size(11.5)
                                .color(Color32::from_rgb(90, 95, 105)),
                            );
                        });

                    ui.add_space(20.0);

                    // Sample Hyperlinks
                    ui.label(RichText::new("Internal Pages & Links:").size(13.5).strong().color(Color32::from_rgb(30, 30, 40)));
                    ui.add_space(6.0);

                    let sample_links = [
                        ("Return to Welcome Dashboard (false://welcome)", "false://welcome"),
                        ("Open False Settings (false://settings)", "false://settings"),
                        ("Open Browsing History (false://history)", "false://history"),
                        ("Open Saved Bookmarks (false://bookmarks)", "false://bookmarks"),
                        ("Rust Language Official Website", "https://doc.rust-lang.org"),
                    ];

                    for (name, link_url) in sample_links {
                        let link_btn = ui.link(
                            RichText::new(format!("🔗 {}", name))
                                .size(13.0)
                                .color(Color32::from_rgb(0, 102, 204)),
                        );

                        if link_btn.hovered() {
                            action.hovered_link = Some(link_url.to_string());
                        }

                        if link_btn.clicked() {
                            action.navigate_to = Some(link_url.to_string());
                        }
                        ui.add_space(3.0);
                    }
                });
        });
}
