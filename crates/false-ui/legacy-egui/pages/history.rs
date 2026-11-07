//! False Browser History Internal Webapp (`false://history`).

use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Ui, Vec2};
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct HistoryRecord {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub host: String,
    pub time_ago: String,
}

pub struct HistoryState {
    pub search_query: String,
    pub records: Vec<HistoryRecord>,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            records: vec![
                HistoryRecord {
                    id: 1,
                    title: "Welcome to False".into(),
                    url: "false://welcome".into(),
                    host: "false://welcome".into(),
                    time_ago: "Just now".into(),
                },
                HistoryRecord {
                    id: 2,
                    title: "Acid1 CSS 1 Test Suite".into(),
                    url: "https://www.w3.org/Style/CSS/Test/CSS1/current/test5526c.htm".into(),
                    host: "w3.org".into(),
                    time_ago: "12m ago".into(),
                },
                HistoryRecord {
                    id: 3,
                    title: "Rust Programming Language".into(),
                    url: "https://www.rust-lang.org".into(),
                    host: "rust-lang.org".into(),
                    time_ago: "45m ago".into(),
                },
                HistoryRecord {
                    id: 4,
                    title: "Wikipedia, the free encyclopedia".into(),
                    url: "https://en.wikipedia.org".into(),
                    host: "wikipedia.org".into(),
                    time_ago: "2h ago".into(),
                },
                HistoryRecord {
                    id: 5,
                    title: "Hacker News".into(),
                    url: "https://news.ycombinator.com".into(),
                    host: "news.ycombinator.com".into(),
                    time_ago: "Yesterday".into(),
                },
            ],
        }
    }
}

pub fn render_history_page(
    ui: &mut Ui,
    state: &mut HistoryState,
) -> Option<String> {
    let mut target_navigation = None;

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("Browsing History").size(24.0).strong().color(Theme::FG));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(RichText::new("Clear All History").color(Theme::WARN_AMBER)).clicked() {
                state.records.clear();
            }
        });
    });

    ui.add_space(12.0);

    // Search Filter Bar (matching Familiar .hsearch)
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
                        .hint_text("Search browsing history...")
                        .desired_width(f32::INFINITY)
                        .text_color(Theme::FG)
                );
            });
        });

    ui.add_space(16.0);

    // List of History Entries
    egui::ScrollArea::vertical()
        .id_salt("history_list_scroll")
        .show(ui, |ui| {
            let filter = state.search_query.to_lowercase();
            let mut to_delete = None;

            for (idx, record) in state.records.iter().enumerate() {
                if !filter.is_empty()
                    && !record.title.to_lowercase().contains(&filter)
                    && !record.url.to_lowercase().contains(&filter)
                {
                    continue;
                }

                Frame::new()
                    .fill(Theme::B1)
                    .stroke(Theme::border_stroke())
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);

                            // Favicon badge
                            let icon = if record.url.starts_with("https://") { "🔒" } else { "⚡" };
                            ui.label(RichText::new(icon).size(13.0));

                            // Title & Host
                            ui.vertical(|ui| {
                                let title_btn = ui.link(
                                    RichText::new(&record.title)
                                        .size(13.0)
                                        .strong()
                                        .color(Theme::FG)
                                );
                                if title_btn.clicked() {
                                    target_navigation = Some(record.url.clone());
                                }

                                ui.label(
                                    RichText::new(&record.host)
                                        .size(11.0)
                                        .color(Theme::MUTE)
                                );
                            });

                            // Right info: time ago and delete button
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("×").size(13.0).color(Theme::DIM)).clicked() {
                                    to_delete = Some(idx);
                                }
                                ui.label(RichText::new(&record.time_ago).size(11.0).color(Theme::MUTE));
                            });
                        });
                    });

                ui.add_space(4.0);
            }

            if let Some(del_idx) = to_delete {
                state.records.remove(del_idx);
            }
        });

    target_navigation
}
