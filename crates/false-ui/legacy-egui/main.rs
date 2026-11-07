//! Entry point for the False web browser application.

mod app;
mod chrome;
mod devtools;
mod pages;
mod sidebar;
mod tab;
mod theme;
mod viewport;

use app::FalseBrowserApp;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 860.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("False Browser — Pure Rust Engine"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "False Browser",
        options,
        Box::new(|cc| Ok(Box::new(FalseBrowserApp::new(cc)))),
    )
}
