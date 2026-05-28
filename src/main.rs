#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod models;
mod state;
mod storage;
mod ui;

use eframe::egui;
use std::sync::Arc;
use storage::macro_store;

fn main() -> eframe::Result<()> {
    let config = macro_store::load_config();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    let initial_size = if config.start_compact {
        [480.0, 52.0]
    } else {
        [780.0, 560.0]
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Repeatable")
        .with_inner_size(initial_size)
        .with_decorations(!config.start_compact)
        .with_resizable(!config.start_compact);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    if config.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Repeatable",
        options,
        Box::new(move |cc| Box::new(app::RepeatableApp::new(cc, runtime, config))),
    )
}

fn load_app_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("assets/logo.png");
    eframe::icon_data::from_png_bytes(bytes).ok()
}
