#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod color;
mod drawing;
mod gui_update;
mod history;
mod history_view;
mod icon;
mod palette;
mod picker_overlay;
mod screen;
mod update;
mod window_placement;

use anyhow::Result;
use eframe::egui;

const APP_WINDOW_WIDTH: f32 = 568.0;
const APP_WINDOW_HEIGHT: f32 = 920.0;

fn main() -> Result<()> {
    let window_size = egui::vec2(APP_WINDOW_WIDTH, APP_WINDOW_HEIGHT);
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(app::APP_TITLE)
            .with_app_id("com.thevelasquez.v_color_picker")
            .with_inner_size(window_size)
            .with_min_inner_size(window_size)
            .with_resizable(true)
            .with_icon(icon::app_icon()),
        centered: false,
        persist_window: true,
        ..Default::default()
    };

    eframe::run_native(
        app::APP_TITLE,
        native_options,
        Box::new(|cc| Ok(Box::new(app::VColorPickerApp::new(cc)))),
    )
    .map_err(|error| anyhow::anyhow!("{error}"))
}
