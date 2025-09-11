use crate::conversion::{ConversionMode, ConversionSettings};
use eframe::egui;
use serde_json;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveTab {
    Basic,
    Advanced,
    Progress,
}

impl Default for ActiveTab {
    fn default() -> Self {
        ActiveTab::Basic
    }
}

pub struct FFmpegApp {
    pub input_file: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub settings: ConversionSettings,
    pub active_tab: ActiveTab,
    pub is_converting: bool,
    pub progress: f32,
    pub error: Option<String>,
    pub status_message: String,
    pub conversion_receiver: Option<mpsc::Receiver<ConversionMessage>>,
}

#[derive(Debug)]
enum ConversionMessage {
    Progress(f32),
    Completed,
    Error(String),
}

impl Default for FFmpegApp {
    fn default() -> Self {
        Self::new()
    }
}

impl FFmpegApp {
    pub fn new() -> Self {
        Self {
            input_file: None,
            output_file: None,
            settings: ConversionSettings::default(),
            active_tab: ActiveTab::Basic,
            is_converting: false,
            progress: 0.0,
            error: None,
            status_message: "Ready".to_string(),
            conversion_receiver: None,
        }
    }

    pub fn select_input_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Video Files", &["mp4", "mkv", "avi", "mov", "webm", "flv"])
            .add_filter("Audio Files", &["mp3", "wav", "flac", "aac", "ogg"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.input_file = Some(path);

            if self.output_file.is_none() {
                self.auto_generate_output();
            }

            self.status_message = "Input file selected".to_string();
            self.clear_error();
        }
    }

    pub fn select_output_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&format!("output.{}", self.settings.container))
            .save_file()
        {
            self.output_file = Some(path);
            self.status_message = "Output file selected".to_string();
        }
    }

    pub fn select_output_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.output_file = Some(path);
            self.status_message = "Output folder selected".to_string();
        }
    }

    fn auto_generate_output(&mut self) {
        if let Some(input) = &self.input_file {
            if let Some(parent) = input.parent() {
                if let Some(stem) = input.file_stem() {
                    let suffix = match self.settings.mode {
                        ConversionMode::Remux => "_remux",
                        ConversionMode::Convert => "_converted",
                    };

                    let filename = format!(
                        "{}{}.{}",
                        stem.to_string_lossy(),
                        suffix,
                        self.settings.container
                    );

                    let output_path = parent.join(filename);
                    self.output_file = Some(output_path);
                    self.status_message = "Output auto-generated".to_string();
                }
            }
        }
    }

    pub fn can_start_conversion(&self) -> bool {
        self.input_file.is_some() && !self.is_converting
    }

    pub fn start_conversion(&mut self) {
        if !self.can_start_conversion() {
            return;
        }

        let input_file = self.input_file.as_ref().unwrap().clone();
        let output_file = if let Some(output) = &self.output_file {
            if output.is_dir() || (!output.exists() && output.extension().is_none()) {
                if let Some(stem) = input_file.file_stem() {
                    let suffix = match self.settings.mode {
                        ConversionMode::Remux => "_remux",
                        ConversionMode::Convert => "_converted",
                    };
                    let filename = format!(
                        "{}{}.{}",
                        stem.to_string_lossy(),
                        suffix,
                        self.settings.container
                    );
                    output.join(filename)
                } else {
                    return;
                }
            } else {
                output.clone()
            }
        } else {
            if let Some(parent) = input_file.parent() {
                if let Some(stem) = input_file.file_stem() {
                    let suffix = match self.settings.mode {
                        ConversionMode::Remux => "_remux",
                        ConversionMode::Convert => "_converted",
                    };
                    let filename = format!(
                        "{}{}.{}",
                        stem.to_string_lossy(),
                        suffix,
                        self.settings.container
                    );
                    parent.join(filename)
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let settings = self.settings.clone();
        let (tx, rx) = mpsc::channel();
        self.conversion_receiver = Some(rx);

        thread::spawn(move || {
            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-y");

            // Hardware acceleration must come before input
            if settings.use_hardware_accel && settings.mode == ConversionMode::Convert {
                cmd.arg("-hwaccel").arg("auto");
            }

            cmd.arg("-i").arg(&input_file);

            match settings.mode {
                ConversionMode::Remux => {
                    // Fast remux - copy streams without re-encoding
                    cmd.arg("-c").arg("copy");
                }
                ConversionMode::Convert => {
                    // Video codec
                    if settings.video_codec == "copy" {
                        cmd.arg("-c:v").arg("copy");
                    } else {
                        cmd.arg("-c:v").arg(&settings.video_codec);

                        // Add codec-specific quality settings
                        if settings.video_codec.contains("264")
                            || settings.video_codec.contains("265")
                        {
                            cmd.arg("-crf").arg(&settings.quality);
                            cmd.arg("-preset").arg("medium");
                        } else if settings.video_codec.contains("vpx") {
                            cmd.arg("-crf").arg(&settings.quality);
                            cmd.arg("-b:v").arg("0"); // VBR mode
                        }

                        // Hardware acceleration for specific codecs
                        if settings.use_hardware_accel
                            && (settings.video_codec.contains("264")
                                || settings.video_codec.contains("265"))
                        {
                            // Hardware acceleration already set above
                        }
                    }

                    // Audio codec
                    if settings.audio_codec == "copy" {
                        cmd.arg("-c:a").arg("copy");
                    } else {
                        cmd.arg("-c:a").arg(&settings.audio_codec);

                        // Add audio quality settings
                        match settings.audio_codec.as_str() {
                            "aac" => {
                                cmd.arg("-b:a").arg("128k");
                            }
                            "libmp3lame" => {
                                cmd.arg("-b:a").arg("192k");
                            }
                            "libopus" => {
                                cmd.arg("-b:a").arg("128k");
                            }
                            "flac" => {
                                // FLAC is lossless, no bitrate setting needed
                            }
                            codec if codec.starts_with("pcm_") => {
                                // PCM is uncompressed, no additional settings needed
                            }
                            _ => {}
                        }
                    }

                    // Container-specific optimizations
                    match settings.container.as_str() {
                        "mov" | "mp4" => {
                            cmd.arg("-movflags").arg("faststart");
                        }
                        "webm" => {
                            // Ensure compatible codecs for WebM
                            if !settings.video_codec.contains("vp")
                                && settings.video_codec != "copy"
                            {
                                // WebM should use VP codecs, but don't override user choice
                            }
                            if settings.audio_codec == "aac"
                                || settings.audio_codec.starts_with("pcm_")
                            {
                                // WebM doesn't support AAC or PCM well, but don't force change
                            }
                        }
                        "wav" => {
                            // WAV is ideal for PCM audio
                        }
                        _ => {}
                    }
                }
            }

            cmd.arg(&output_file);

            println!("DEBUG: Running FFmpeg command: {:?}", cmd);

            let _ = tx.send(ConversionMessage::Progress(0.0));

            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        let _ = tx.send(ConversionMessage::Progress(100.0));
                        let _ = tx.send(ConversionMessage::Completed);
                    } else {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        println!("DEBUG: FFmpeg stderr: {}", error_msg);
                        let _ = tx.send(ConversionMessage::Error(format!(
                            "FFmpeg failed: {}",
                            error_msg
                        )));
                    }
                }
                Err(e) => {
                    println!("DEBUG: Failed to run FFmpeg: {}", e);
                    let _ = tx.send(ConversionMessage::Error(format!(
                        "Failed to run FFmpeg: {}",
                        e
                    )));
                }
            }
        });

        self.is_converting = true;
        self.progress = 0.0;
        self.status_message = "Starting conversion...".to_string();
        self.clear_error();
    }

    pub fn cancel_conversion(&mut self) {
        self.is_converting = false;
        self.status_message = "Conversion cancelled".to_string();
    }

    pub fn update_conversion_status(&mut self) {
        let mut should_clear_receiver = false;
        let mut messages = Vec::new();

        if let Some(receiver) = &self.conversion_receiver {
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }

        for message in messages {
            match message {
                ConversionMessage::Progress(progress) => {
                    self.progress = progress;
                    if progress < 100.0 {
                        self.status_message = format!("Converting... {:.1}%", progress);
                    }
                }
                ConversionMessage::Completed => {
                    self.is_converting = false;
                    self.progress = 100.0;
                    self.status_message = "Conversion completed successfully!".to_string();
                    should_clear_receiver = true;
                }
                ConversionMessage::Error(error) => {
                    self.is_converting = false;
                    self.error = Some(error);
                    self.status_message = "Conversion failed".to_string();
                    should_clear_receiver = true;
                }
            }
        }

        if should_clear_receiver {
            self.conversion_receiver = None;
        }
    }

    pub fn apply_preset(&mut self, preset_name: &str) {
        match preset_name {
            "Web Standard (H.264/MP4)" => {
                self.settings.mode = ConversionMode::Convert;
                self.settings.container = "mp4".to_string();
                self.settings.video_codec = "libx264".to_string();
                self.settings.audio_codec = "aac".to_string();
                self.settings.quality = "23".to_string();
                self.settings.use_hardware_accel = true;
            }
            "High Quality (H.265/MKV)" => {
                self.settings.mode = ConversionMode::Convert;
                self.settings.container = "mkv".to_string();
                self.settings.video_codec = "libx265".to_string();
                self.settings.audio_codec = "flac".to_string();
                self.settings.quality = "20".to_string();
                self.settings.use_hardware_accel = true;
            }
            "Small File Size (H.265)" => {
                self.settings.mode = ConversionMode::Convert;
                self.settings.container = "mp4".to_string();
                self.settings.video_codec = "libx265".to_string();
                self.settings.audio_codec = "aac".to_string();
                self.settings.quality = "28".to_string();
                self.settings.use_hardware_accel = true;
            }
            "Professional PCM Archive" => {
                self.settings.mode = ConversionMode::Convert;
                self.settings.container = "mov".to_string();
                self.settings.video_codec = "copy".to_string();
                self.settings.audio_codec = "pcm_s16le".to_string();
                self.settings.quality = "18".to_string();
                self.settings.use_hardware_accel = false;
            }
            "Fast Remux to MP4" => {
                self.settings.mode = ConversionMode::Remux;
                self.settings.container = "mp4".to_string();
            }
            "Fast Remux to MOV" => {
                self.settings.mode = ConversionMode::Remux;
                self.settings.container = "mov".to_string();
            }
            "Fast Remux to MKV" => {
                self.settings.mode = ConversionMode::Remux;
                self.settings.container = "mkv".to_string();
            }
            "Fast Remux to WebM" => {
                self.settings.mode = ConversionMode::Remux;
                self.settings.container = "webm".to_string();
            }
            _ => {}
        }
        self.update_output_extension();
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn get_input_file(&self) -> Option<&PathBuf> {
        self.input_file.as_ref()
    }

    pub fn get_output_file(&self) -> Option<&PathBuf> {
        self.output_file.as_ref()
    }

    pub fn get_settings(&self) -> &ConversionSettings {
        &self.settings
    }

    pub fn get_settings_mut(&mut self) -> &mut ConversionSettings {
        &mut self.settings
    }

    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    pub fn get_state(&self) -> serde_json::Value {
        serde_json::json!({
            "input_file": self.input_file.as_ref().map(|p| p.to_string_lossy()),
            "output_file": self.output_file.as_ref().map(|p| p.to_string_lossy()),
            "settings": {
                "mode": format!("{:?}", self.settings.mode),
                "container": self.settings.container,
                "video_codec": self.settings.video_codec,
                "audio_codec": self.settings.audio_codec,
                "quality": self.settings.quality,
                "use_hardware_accel": self.settings.use_hardware_accel,
            },
            "is_converting": self.is_converting,
            "progress": self.progress,
            "status": self.status_message,
        })
    }

    pub fn update_output_extension(&mut self) {
        if let Some(output) = &self.output_file {
            if output.is_file() || output.extension().is_some() {
                if let Some(parent) = output.parent() {
                    if let Some(stem) = output.file_stem() {
                        let new_path = parent.join(format!(
                            "{}.{}",
                            stem.to_string_lossy(),
                            self.settings.container
                        ));
                        self.output_file = Some(new_path);
                    }
                }
            }
        } else {
            self.auto_generate_output();
        }
    }
}

impl eframe::App for FFmpegApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_conversion_status();

        // Minimal dark theme - only black, white, and blue accent
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.window_fill = egui::Color32::BLACK;
        style.visuals.panel_fill = egui::Color32::from_gray(8);
        style.visuals.faint_bg_color = egui::Color32::from_gray(15);
        style.visuals.extreme_bg_color = egui::Color32::from_gray(5);

        // Interactive elements - only grays and blue
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_gray(20);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(25);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(35);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_gray(45);

        // Only blue for accents
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(70, 130, 180);
        style.visuals.selection.stroke.color = egui::Color32::from_rgb(100, 150, 200);

        style.spacing.button_padding = egui::vec2(16.0, 10.0);
        style.spacing.item_spacing = egui::vec2(12.0, 8.0);

        ctx.set_style(style);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                // Add scrollable area for all content
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(20.0);

                        // Title section
                        ui.vertical_centered(|ui| {
                            ui.heading(
                                egui::RichText::new("FFmpeg Pro")
                                    .size(36.0)
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            );
                            ui.add_space(5.0);
                            ui.label(
                                egui::RichText::new("Professional Video Conversion")
                                    .size(14.0)
                                    .color(egui::Color32::GRAY),
                            );
                        });

                        ui.add_space(30.0);

                        // File selection section
                        self.render_file_section(ui);
                        ui.add_space(20.0);

                        // Mode and settings
                        self.render_mode_settings_section(ui);
                        ui.add_space(20.0);

                        // Status and controls
                        self.render_status_section(ui);
                        ui.add_space(20.0);

                        // Presets
                        self.render_presets_section(ui);
                        ui.add_space(30.0);
                    });
            });

        if self.is_converting {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
    }
}

impl FFmpegApp {
    fn create_section_frame() -> egui::Frame {
        egui::Frame::none()
            .fill(egui::Color32::from_gray(12))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(30)))
            .rounding(8.0)
            .inner_margin(20.0)
    }

    fn render_file_section(&mut self, ui: &mut egui::Ui) {
        Self::create_section_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("Files")
                    .size(18.0)
                    .color(egui::Color32::WHITE)
                    .strong(),
            );
            ui.add_space(15.0);

            // Input file
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [120.0, 35.0],
                        egui::Button::new("Select Input").rounding(6.0),
                    )
                    .clicked()
                {
                    self.select_input_file();
                }

                ui.add_space(15.0);

                if let Some(input) = &self.input_file {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(
                                input.file_name().unwrap_or_default().to_string_lossy(),
                            )
                            .size(14.0)
                            .color(egui::Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new(input.parent().unwrap_or(input).to_string_lossy())
                                .size(11.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                } else {
                    ui.label(egui::RichText::new("No file selected").color(egui::Color32::GRAY));
                }
            });

            ui.add_space(10.0);

            // Output selection
            ui.horizontal(|ui| {
                if ui
                    .add_sized([60.0, 32.0], egui::Button::new("File").rounding(6.0))
                    .clicked()
                {
                    self.select_output_file();
                }

                if ui
                    .add_sized([70.0, 32.0], egui::Button::new("Folder").rounding(6.0))
                    .clicked()
                {
                    self.select_output_folder();
                }

                ui.add_space(15.0);

                if let Some(output) = &self.output_file {
                    ui.label(
                        egui::RichText::new(format!(
                            "Output: {}",
                            output.file_name().unwrap_or_default().to_string_lossy()
                        ))
                        .size(12.0)
                        .color(egui::Color32::LIGHT_GRAY),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Auto-generated in input folder")
                            .size(12.0)
                            .color(egui::Color32::GRAY),
                    );
                }
            });
        });
    }

    fn render_mode_settings_section(&mut self, ui: &mut egui::Ui) {
        Self::create_section_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("Settings")
                    .size(18.0)
                    .color(egui::Color32::WHITE)
                    .strong(),
            );
            ui.add_space(15.0);

            // Mode selection
            ui.horizontal(|ui| {
                ui.label("Mode:");
                ui.add_space(10.0);

                let convert_selected = self.settings.mode == ConversionMode::Convert;
                let remux_selected = self.settings.mode == ConversionMode::Remux;

                if ui
                    .add_sized(
                        [100.0, 32.0],
                        egui::Button::new("Convert")
                            .fill(if convert_selected {
                                egui::Color32::from_rgb(70, 130, 180)
                            } else {
                                egui::Color32::from_gray(40)
                            })
                            .rounding(6.0),
                    )
                    .clicked()
                {
                    self.settings.mode = ConversionMode::Convert;
                }

                if ui
                    .add_sized(
                        [100.0, 32.0],
                        egui::Button::new("Remux")
                            .fill(if remux_selected {
                                egui::Color32::from_rgb(70, 130, 180)
                            } else {
                                egui::Color32::from_gray(40)
                            })
                            .rounding(6.0),
                    )
                    .clicked()
                {
                    self.settings.mode = ConversionMode::Remux;
                }
            });

            ui.add_space(12.0);

            // Container
            ui.horizontal(|ui| {
                ui.label("Container:");
                ui.add_space(10.0);

                let containers = ["mp4", "mkv", "mov", "webm", "wav", "avi"];
                for container in containers {
                    let selected = self.settings.container == container;
                    if ui
                        .add_sized(
                            [50.0, 28.0],
                            egui::Button::new(container.to_uppercase())
                                .fill(if selected {
                                    egui::Color32::from_gray(60)
                                } else {
                                    egui::Color32::from_gray(35)
                                })
                                .rounding(4.0),
                        )
                        .clicked()
                    {
                        self.settings.container = container.to_string();
                        self.update_output_extension();
                    }
                }
            });

            if self.settings.mode == ConversionMode::Convert {
                ui.add_space(12.0);

                // Video codec
                ui.horizontal(|ui| {
                    ui.label("Video:");
                    ui.add_space(20.0);

                    let video_codecs = [
                        ("libx264", "H.264"),
                        ("libx265", "H.265"),
                        ("libvpx-vp9", "VP9"),
                        ("copy", "Copy"),
                    ];

                    for (codec, label) in video_codecs {
                        let selected = self.settings.video_codec == codec;
                        if ui
                            .add_sized(
                                [70.0, 28.0],
                                egui::Button::new(label)
                                    .fill(if selected {
                                        egui::Color32::from_gray(60)
                                    } else {
                                        egui::Color32::from_gray(35)
                                    })
                                    .rounding(4.0),
                            )
                            .clicked()
                        {
                            self.settings.video_codec = codec.to_string();
                        }
                    }
                });

                ui.add_space(8.0);

                // Audio codec
                ui.horizontal(|ui| {
                    ui.label("Audio:");
                    ui.add_space(20.0);

                    let audio_codecs = [
                        ("aac", "AAC"),
                        ("libmp3lame", "MP3"),
                        ("flac", "FLAC"),
                        ("pcm_s16le", "PCM"),
                        ("copy", "Copy"),
                    ];

                    for (codec, label) in audio_codecs {
                        let selected = self.settings.audio_codec == codec;
                        if ui
                            .add_sized(
                                [60.0, 28.0],
                                egui::Button::new(label)
                                    .fill(if selected {
                                        egui::Color32::from_gray(60)
                                    } else {
                                        egui::Color32::from_gray(35)
                                    })
                                    .rounding(4.0),
                            )
                            .clicked()
                        {
                            self.settings.audio_codec = codec.to_string();
                        }
                    }
                });

                if self.settings.video_codec != "copy" {
                    ui.add_space(8.0);

                    // Quality
                    ui.horizontal(|ui| {
                        ui.label("Quality:");
                        ui.add_space(10.0);

                        let qualities = [
                            ("18", "Perfect"),
                            ("23", "High"),
                            ("26", "Medium"),
                            ("28", "Small"),
                        ];
                        for (value, label) in qualities {
                            let selected = self.settings.quality == value;
                            if ui
                                .add_sized(
                                    [70.0, 28.0],
                                    egui::Button::new(label)
                                        .fill(if selected {
                                            egui::Color32::from_gray(60)
                                        } else {
                                            egui::Color32::from_gray(35)
                                        })
                                        .rounding(4.0),
                                )
                                .clicked()
                            {
                                self.settings.quality = value.to_string();
                            }
                        }
                    });
                }

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.settings.use_hardware_accel,
                        "Hardware Acceleration",
                    );
                });
            } else {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Fast remuxing - no quality loss")
                        .size(12.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
            }
        });
    }

    fn render_status_section(&mut self, ui: &mut egui::Ui) {
        Self::create_section_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("Status & Controls")
                    .size(18.0)
                    .color(egui::Color32::WHITE)
                    .strong(),
            );
            ui.add_space(15.0);

            let status_color = if self.is_converting {
                egui::Color32::YELLOW
            } else if self.error.is_some() {
                egui::Color32::LIGHT_RED
            } else {
                egui::Color32::LIGHT_GREEN
            };

            ui.label(
                egui::RichText::new(&self.status_message)
                    .size(14.0)
                    .color(status_color),
            );

            if self.is_converting || self.progress > 0.0 {
                ui.add_space(10.0);
                let progress_bar = egui::ProgressBar::new(self.progress / 100.0)
                    .text(format!("{:.1}%", self.progress))
                    .desired_height(20.0);
                ui.add(progress_bar);
            }

            if let Some(error) = &self.error {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!("Error: {}", error))
                        .size(12.0)
                        .color(egui::Color32::LIGHT_RED),
                );
                if ui
                    .add_sized([60.0, 24.0], egui::Button::new("Clear").rounding(4.0))
                    .clicked()
                {
                    self.clear_error();
                }
            }

            ui.add_space(15.0);

            // Main control button
            ui.horizontal(|ui| {
                if self.is_converting {
                    if ui
                        .add_sized(
                            [120.0, 40.0],
                            egui::Button::new("Cancel")
                                .fill(egui::Color32::from_gray(80))
                                .rounding(8.0),
                        )
                        .clicked()
                    {
                        self.cancel_conversion();
                    }
                } else {
                    let can_convert = self.can_start_conversion();
                    let button_text = if can_convert {
                        "Start Conversion"
                    } else {
                        "Select Input File"
                    };

                    ui.add_enabled_ui(can_convert, |ui| {
                        if ui
                            .add_sized(
                                [150.0, 40.0],
                                egui::Button::new(button_text)
                                    .fill(egui::Color32::from_rgb(70, 130, 180))
                                    .rounding(8.0),
                            )
                            .clicked()
                        {
                            self.start_conversion();
                        }
                    });
                }
            });
        });
    }

    fn render_presets_section(&mut self, ui: &mut egui::Ui) {
        Self::create_section_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("Quick Presets")
                    .size(18.0)
                    .color(egui::Color32::WHITE)
                    .strong(),
            );
            ui.add_space(15.0);

            // Convert presets
            ui.label("Convert:");
            ui.add_space(5.0);
            ui.horizontal_wrapped(|ui| {
                let convert_presets = [
                    ("Web Standard (H.264/MP4)", "Web"),
                    ("High Quality (H.265/MKV)", "High Quality"),
                    ("Small File Size (H.265)", "Small File"),
                    ("Professional PCM Archive", "Archive"),
                ];

                for (preset, label) in convert_presets {
                    if ui
                        .add_sized(
                            [100.0, 32.0],
                            egui::Button::new(label)
                                .fill(egui::Color32::from_gray(45))
                                .rounding(6.0),
                        )
                        .clicked()
                    {
                        self.apply_preset(preset);
                    }
                }
            });
        });
    }
}
