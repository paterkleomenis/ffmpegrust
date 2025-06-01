mod app;
mod config;
mod conversion;
mod ui;

use app::FFmpegApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("FFmpeg Converter Pro"),
        ..Default::default()
    };

    eframe::run_native(
        "FFmpeg Converter Pro",
        options,
        Box::new(|_cc| Box::new(FFmpegApp::new())),
    )
}