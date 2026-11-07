//! False Browser Bookmarks Internal Webapp (`false://bookmarks`).

use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Ui, Vec2};
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct BookmarkItem {
    pub title: String,
    pub url: String,
    pub host: String,
}

pub struct BookmarksState {
    pub search_query: String,
    pub items: Vec<BookmarkItem>,
}

impl Default for BookmarksState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            items: vec![
                BookmarkItem {
                    title: "Welcome to False".into(),
                    url: "false://welcome".into(),
                    host: "false://welcome".into(),
                },
                BookmarkItem {
                    title: "Acid1 CSS Test Suite".into(),
                    url: "https://www.w3.org/Style/CSS/Test/CSS1/current/test5526c.htm".into(),
                    host: "w3.org".into(),
                },
                BookmarkItem {
                    title: "The Rust Programming Language".into(),
                    url: "https://www.rust-lang.org".into(),
                    host: "rust-lang.org".into(),
                },
                BookmarkItem {
                    title: "Wikipedia, the free encyclopedia".into(),
                    url: "https://en.wikipedia.org".into(),
                    host: "wikipedia.org".into(),
                },
                BookmarkItem {
                    title: "Hacker News".into(),
                    url: "https://news.ycombinator.com".into(),
                    host: "news.ycombinator.com".into(),
                },
                BookmarkItem {
                    title: "DuckDuckGo Privacy Search".into(),
                    url: "https://duckduckgo.com".into(),
                    host: "duckduckgo.com".into(),
                },
            ],
        }
    }
}

pub fn render_bookmarks_page(
    ui: &mut Ui,
    state: &mut BookmarksState,
) -> Option<String> {
    let mut target_navigation = None;

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("Saved Bookmarks").size(24.0).strong().color(Theme::FG));
    });

    ui.add_space(12.0);

    // Search input
    Frame::new()
        .fill(Theme::B3)
        .stroke(Theme::border_stroke())
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(13.0).color(Theme::MUTE));
                ui.add(
                    egui::TextEdit::singleline(&mut state.search_query)
                        .hint_text("Search saved bookmarks...")
                        .desired_width(f32::INFINITY)
                        .text_color(Theme::FG)
                );
            });
        });

    ui.add_space(16.0);

    // Grid of bookmarks
    egui::ScrollArea::vertical()
        .id_salt("bookmarks_grid_scroll")
        .show(ui, |ui| {
            let filter = state.search_query.to_lowercase();
            let mut to_delete = None;

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(12.0, 12.0);

                for (idx, item) in state.items.iter().enumerate() {
                    if !filter.is_empty()
                        && !item.title.to_lowercase().contains(&filter)
                        && !item.url.to_lowercase().contains(&filter)
                    {
                        continue;
                    }

                    Frame::new()
                        .fill(Theme::B1)
                        .stroke(Theme::border_stroke())
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(14))
                        .show(ui, |ui| {
                            ui.set_width(260.0);
                            ui.set_height(75.0);

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("★").color(Theme::STAR_YELLOW).size(13.0));
                                    let btn = ui.link(
                                        RichText::new(&item.title)
                                            .size(13.0)
                                            .strong()
                                            .color(Theme::FG)
                                    );
                                    if btn.clicked() {
                                        target_navigation = Some(item.url.clone());
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button(RichText::new("×").size(12.0).color(Theme::DIM)).clicked() {
                                            to_delete = Some(idx);
                                        }
                                    });
                                });

                                ui.add_space(4.0);
                                ui.label(RichText::new(&item.host).size(11.0).color(Theme::MUTE));
                            });
                        });
                }
            });

            if let Some(del_idx) = to_delete {
                state.items.remove(del_idx);
            }
        });

    target_navigation
}
