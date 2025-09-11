use tokio::runtime::Runtime;
use tracing_subscriber;

mod app;
mod config;
mod constants;
mod conversion;
mod events;
mod ffmpeg_installer;
mod presets;
mod security;
mod services;
mod state;
mod updater;

// Window constants now defined inline
use app::FFmpegApp;

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("Starting FFmpeg Converter Pro");

    // Create tokio runtime for async operations
    let _rt = Runtime::new().expect("Failed to create async runtime");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1200.0, 700.0])
            .with_title("FFmpeg Pro - Video Conversion Studio")
            .with_resizable(true)
            .with_maximize_button(true),
        ..Default::default()
    };

    // Create the application with the async runtime
    let app_creator =
        move |_cc: &eframe::CreationContext| -> Box<dyn eframe::App> { Box::new(FFmpegApp::new()) };

    let result = eframe::run_native("FFmpeg Pro", options, Box::new(app_creator));

    tracing::info!("Application shutting down");
    result
}
