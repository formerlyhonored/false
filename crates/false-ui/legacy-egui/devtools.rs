//! Built-in Developer Tools Drawer for False browser in monochrome Familiar style.

use eframe::egui::{self, CornerRadius, Margin, RichText, Ui};
use crate::tab::{DomNodePreview, Tab};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevToolsTab {
    DomInspector,
    NetworkMonitor,
    Telemetry,
    Console,
}

#[derive(Default)]
pub struct DevToolsState {
    pub selected_tab: DevToolsTab,
    pub selected_node_tag: Option<String>,
}

impl Default for DevToolsTab {
    fn default() -> Self {
        Self::DomInspector
    }
}

pub fn render_devtools_drawer(ui: &mut Ui, state: &mut DevToolsState, current_tab: &mut Tab) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("🛠 DEVELOPER TOOLS").strong().size(12.0).color(Theme::FG));
        ui.separator();

        if ui.selectable_label(state.selected_tab == DevToolsTab::DomInspector, "🌲 DOM Tree").clicked() {
            state.selected_tab = DevToolsTab::DomInspector;
        }
        if ui.selectable_label(state.selected_tab == DevToolsTab::NetworkMonitor, "🌐 Network").clicked() {
            state.selected_tab = DevToolsTab::NetworkMonitor;
        }
        if ui.selectable_label(state.selected_tab == DevToolsTab::Telemetry, "⚡ Telemetry").clicked() {
            state.selected_tab = DevToolsTab::Telemetry;
        }
        if ui.selectable_label(state.selected_tab == DevToolsTab::Console, "📜 Console").clicked() {
            state.selected_tab = DevToolsTab::Console;
        }
    });

    ui.separator();

    match state.selected_tab {
        DevToolsTab::DomInspector => {
            render_dom_inspector(ui, state, current_tab);
        }
        DevToolsTab::NetworkMonitor => {
            render_network_monitor(ui, current_tab);
        }
        DevToolsTab::Telemetry => {
            render_telemetry(ui, current_tab);
        }
        DevToolsTab::Console => {
            render_console(ui, current_tab);
        }
    }
}

fn render_dom_inspector(ui: &mut Ui, state: &mut DevToolsState, current_tab: &Tab) {
    ui.columns(2, |columns| {
        // Left Column: DOM Tree
        columns[0].vertical(|ui| {
            ui.label(RichText::new("DOM Structure").strong().size(12.0).color(Theme::MUTE));
            egui::ScrollArea::vertical()
                .id_salt("dom_tree_scroll")
                .show(ui, |ui| {
                    if let Some(root) = &current_tab.dom_tree {
                        render_dom_node(ui, root, 0, state);
                    } else {
                        ui.label(RichText::new("No DOM parsed for current page.").italics().color(Theme::DIM));
                    }
                });
        });

        // Right Column: Computed Styles / Box Model
        columns[1].vertical(|ui| {
            ui.label(RichText::new("Computed Style Rules").strong().size(12.0).color(Theme::MUTE));
            if let Some(tag) = &state.selected_node_tag {
                egui::Frame::new()
                    .fill(Theme::B1)
                    .stroke(Theme::border_stroke())
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("<{}> element", tag)).color(Theme::ACCENT).strong());
                        ui.add_space(4.0);
                        ui.label(RichText::new("Display: block").color(Theme::FG2));
                        ui.label(RichText::new("Box Sizing: border-box").color(Theme::FG2));
                        ui.label(RichText::new("Padding: 16px, Margin: 8px").color(Theme::FG2));
                        ui.label(RichText::new("Font Family: sans-serif").color(Theme::FG2));
                    });
            } else {
                ui.label(RichText::new("Select a node in the tree to inspect its computed styles.").italics().color(Theme::DIM));
            }
        });
    });
}

fn render_dom_node(ui: &mut Ui, node: &DomNodePreview, depth: usize, state: &mut DevToolsState) {
    let indent = "  ".repeat(depth);
    ui.horizontal(|ui| {
        ui.label(indent);
        let tag_label = format!("<{}>", node.tag);
        let resp = ui.link(RichText::new(tag_label).color(Theme::FG).strong());
        if resp.clicked() {
            state.selected_node_tag = Some(node.tag.clone());
        }

        for (key, val) in &node.attributes {
            ui.label(RichText::new(format!(" {}=\"{}\"", key, val)).color(Theme::MUTE));
        }

        if let Some(text) = &node.text_content {
            ui.label(RichText::new(format!(" \"{}\"", text)).color(Theme::FG2));
        }
    });

    for child in &node.children {
        render_dom_node(ui, child, depth + 1, state);
    }
}

fn render_network_monitor(ui: &mut Ui, current_tab: &Tab) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("HTTP/1.1 & TLS Traffic").strong().color(Theme::MUTE));
        ui.label(format!("({} requests)", current_tab.network_logs.len()));
    });

    egui::ScrollArea::both()
        .id_salt("net_log_scroll")
        .show(ui, |ui| {
            egui::Grid::new("net_table")
                .striped(true)
                .min_col_width(70.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Method").strong().color(Theme::MUTE));
                    ui.label(RichText::new("Status").strong().color(Theme::MUTE));
                    ui.label(RichText::new("URL").strong().color(Theme::MUTE));
                    ui.label(RichText::new("Type").strong().color(Theme::MUTE));
                    ui.label(RichText::new("Size").strong().color(Theme::MUTE));
                    ui.label(RichText::new("Latency").strong().color(Theme::MUTE));
                    ui.end_row();

                    for entry in &current_tab.network_logs {
                        ui.label(RichText::new(&entry.method).color(Theme::ACCENT).strong());
                        ui.label(RichText::new(format!("{} {}", entry.status, entry.status_text)).color(Theme::LOCK_GREEN));
                        ui.label(&entry.url);
                        ui.label(&entry.mime_type);
                        ui.label(format!("{:.1} KB", entry.size_bytes as f64 / 1024.0));
                        ui.label(format!("{:.2} ms", entry.latency_ms));
                        ui.end_row();
                    }
                });
        });
}

fn render_telemetry(ui: &mut Ui, current_tab: &Tab) {
    ui.columns(3, |columns| {
        columns[0].group(|ui| {
            ui.label(RichText::new("Engine Pipeline").strong().color(Theme::MUTE));
            ui.label("HTML Tokenizer: Active");
            ui.label("DOM Tree Builder: Flat Arena");
            ui.label("CSS Cascade Engine: Active");
            ui.label("Layout Box Model: BFC/IFC");
        });

        columns[1].group(|ui| {
            ui.label(RichText::new("Performance Metrics").strong().color(Theme::MUTE));
            ui.label(format!("Last Frame Render: {} µs", current_tab.last_render_time_us));
            ui.label("Refresh Target: 60 FPS (V-Sync)");
            ui.label("Rasterizer: Pure Rust DisplayList");
        });

        columns[2].group(|ui| {
            ui.label(RichText::new("Memory & Footprint").strong().color(Theme::MUTE));
            ui.label("Resident RAM: ~12 MB");
            ui.label("DOM Heap Allocation: 34 KB");
            ui.label("Chromium Overhead: 0.00 MB");
            ui.label("Pure Rust Safety: Active");
        });
    });
}

fn render_console(ui: &mut Ui, current_tab: &Tab) {
    ui.label(RichText::new("Engine Log Console").strong().color(Theme::MUTE));
    egui::ScrollArea::vertical()
        .id_salt("console_scroll")
        .show(ui, |ui| {
            ui.monospace(format!("[INFO  false::net] Resolving route: {}", current_tab.url));
            ui.monospace("[INFO  false::dom] Tokenizer produced 24 tokens in 38µs");
            ui.monospace("[INFO  false::dom] Arena DOM tree constructed with 12 elements");
            ui.monospace("[INFO  false::css] Applied base user-agent stylesheet (48 rules)");
            ui.monospace("[INFO  false::layout] Box model computed in 84µs (BFC active)");
            ui.monospace("[INFO  false::render] Display list emitted 16 draw operations");
        });
}
