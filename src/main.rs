mod app;
mod cli;
mod color;
mod drawing;
mod history;
mod icon;
mod picker_overlay;
mod screen;
mod update;

use anyhow::Result;
use eframe::egui;

const APP_WINDOW_WIDTH: f32 = 568.0;
const APP_WINDOW_HEIGHT: f32 = 712.0;

fn main() -> Result<()> {
    if cli::handle_cli(std::env::args().skip(1))? {
        return Ok(());
    }

    let window_size = egui::vec2(APP_WINDOW_WIDTH, APP_WINDOW_HEIGHT);
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(app::APP_TITLE)
            .with_app_id("com.thevelasquez.v_color_picker")
            .with_inner_size(window_size)
            .with_min_inner_size(window_size)
            .with_max_inner_size(window_size)
            .with_resizable(false)
            .with_icon(icon::app_icon()),
        centered: true,
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        app::APP_TITLE,
        native_options,
        Box::new(|cc| Ok(Box::new(app::VColorPickerApp::new(cc)))),
    )
    .map_err(|error| anyhow::anyhow!("{error}"))
}
