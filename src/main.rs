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
    let runtime = Arc::new(Runtime::new()?);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("FFmpeg Rust")
            .with_resizable(true),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "FFmpeg Rust",
        options,
        Box::new(move |_cc| Ok(Box::new(FFmpegApp::new(runtime.clone())))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
