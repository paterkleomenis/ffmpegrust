use std::sync::Arc;
use tokio::runtime::Runtime;

mod app;
mod config;
mod conversion;
mod presets;
mod updater;
mod utils;

use app::FFmpegApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create async runtime
    let runtime = Arc::new(Runtime::new()?);

    // Setup GUI options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("FFmpeg Rust")
            .with_resizable(true),
        centered: true,
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "FFmpeg Rust",
        options,
        Box::new(|_cc| Box::new(FFmpegApp::new(runtime))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
