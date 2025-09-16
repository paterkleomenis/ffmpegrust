use crate::config::Config;
use crate::conversion::{
    check_ffmpeg_installation, generate_output_filename, ConversionMessage, ConversionProgress,
    ConversionTask,
};
use crate::presets::{
    AudioCodec, ConversionMode, ConversionPreset, MetadataOptions, PresetManager, VideoCodec,
    VideoFormat,
};
use crate::updater::{UpdateInfo, UpdateStatus, Updater};
use egui::{Align, CentralPanel, Context, Layout, RichText, ScrollArea};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct FFmpegApp {
    // Core state
    config: Config,
    runtime: Arc<Runtime>,

    // File handling
    input_file: Option<PathBuf>,
    output_folder: Option<PathBuf>,

    // Conversion settings
    mode: ConversionMode,
    video_format: VideoFormat,
    video_codec: VideoCodec,
    audio_codec: AudioCodec,

    // Optional settings
    video_bitrate: String,
    audio_bitrate: String,
    resolution: String,
    frame_rate: String,

    // Metadata options
    metadata_options: MetadataOptions,

    // Conversion state
    is_converting: bool,
    progress: Option<ConversionProgress>,
    conversion_receiver: Option<Receiver<ConversionMessage>>,
    status_message: String,
    error_message: Option<String>,

    // Presets
    preset_manager: PresetManager,
    selected_preset: Option<String>,
    new_preset_name: String,
    show_save_preset: bool,

    // Help/Update dialogs
    show_help_dialog: bool,
    show_about_dialog: bool,
    ffmpeg_status: Option<Result<String, String>>,

    // Updater
    updater: Option<Updater>,
    update_status: Option<UpdateStatus>,
    show_update_dialog: bool,
    download_progress_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<f32>>,
    download_progress: f32,
}

impl Default for FFmpegApp {
    fn default() -> Self {
        Self {
            config: Config::load(),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),

            input_file: None,
            output_folder: None,

            mode: ConversionMode::Convert,
            video_format: VideoFormat::Mp4,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,

            video_bitrate: String::new(),
            audio_bitrate: String::new(),
            resolution: String::new(),
            frame_rate: String::new(),

            metadata_options: MetadataOptions::default(),

            is_converting: false,
            progress: None,
            conversion_receiver: None,
            status_message: "Ready".to_string(),
            error_message: None,

            preset_manager: PresetManager::new(),
            selected_preset: None,
            new_preset_name: String::new(),
            show_save_preset: false,

            show_help_dialog: false,
            show_about_dialog: false,
            ffmpeg_status: None,

            updater: None,
            update_status: None,
            show_update_dialog: false,
            download_progress_receiver: None,
            download_progress: 0.0,
        }
    }
}

impl FFmpegApp {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let mut app = Self {
            runtime,
            ..Default::default()
        };

        // Initialize updater
        if let Ok(updater) = Updater::new("1.0.0", "pater/ffmpegrust") {
            app.updater = Some(updater);

            // Check for updates on startup if enabled - disabled by default
            // if app.config.auto_check_updates {
            //     app.check_for_updates();
            // }
        }

        app
    }

    fn select_input_file(&mut self) {
        if let Some(file) = rfd::FileDialog::new()
            .set_title("Select Input Video File")
            .add_filter(
                "Video Files",
                &[
                    "mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "m4v", "3gp", "ts", "mts",
                    "m2ts", "vob", "mpg", "mpeg", "ogv",
                ],
            )
            .set_directory(
                self.config
                    .last_input_folder
                    .as_ref()
                    .unwrap_or(&std::env::current_dir().unwrap_or_default()),
            )
            .pick_file()
        {
            if let Some(parent) = file.parent() {
                self.config.update_input_folder(Some(parent.to_path_buf()));
            }

            self.input_file = Some(file);
            self.error_message = None;
            self.status_message = "Input file selected".to_string();
        }
    }

    fn select_output_folder(&mut self) {
        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Select Output Folder")
            .set_directory(
                self.config
                    .last_output_folder
                    .as_ref()
                    .unwrap_or(&std::env::current_dir().unwrap_or_default()),
            )
            .pick_folder()
        {
            self.config.update_output_folder(Some(folder.clone()));
            self.output_folder = Some(folder);
            self.status_message = "Output folder selected".to_string();
        }
    }

    fn can_start_conversion(&self) -> bool {
        self.input_file.is_some() && self.output_folder.is_some() && !self.is_converting
    }

    fn start_conversion(&mut self) {
        if !self.can_start_conversion() {
            return;
        }

        let input_file = self.input_file.as_ref().unwrap().clone();
        let output_folder = self.output_folder.as_ref().unwrap().clone();

        // Generate output filename
        let output_filename = generate_output_filename(&input_file, &self.video_format);
        let output_file = output_folder.join(output_filename.file_name().unwrap());

        // Create preset from current settings
        let preset = ConversionPreset {
            name: "Current".to_string(),
            mode: self.mode.clone(),
            video_format: self.video_format.clone(),
            video_codec: self.video_codec.clone(),
            audio_codec: self.audio_codec.clone(),
            video_bitrate: if self.video_bitrate.is_empty() {
                None
            } else {
                Some(self.video_bitrate.clone())
            },
            audio_bitrate: if self.audio_bitrate.is_empty() {
                None
            } else {
                Some(self.audio_bitrate.clone())
            },
            resolution: if self.resolution.is_empty() {
                None
            } else {
                Some(self.resolution.clone())
            },
            frame_rate: if self.frame_rate.is_empty() {
                None
            } else {
                Some(self.frame_rate.clone())
            },
            metadata_options: self.metadata_options.clone(),
        };

        // Create communication channel
        let (sender, receiver) = std::sync::mpsc::channel();
        self.conversion_receiver = Some(receiver);

        // Create and start conversion task
        let task = ConversionTask::new(input_file, output_file, preset, sender);

        self.runtime.spawn(async move {
            task.execute().await;
        });

        self.is_converting = true;
        self.progress = None;
        self.error_message = None;
        self.status_message = "Starting conversion...".to_string();
    }

    fn stop_conversion(&mut self) {
        self.is_converting = false;
        self.progress = None;
        self.conversion_receiver = None;
        self.status_message = "Conversion stopped".to_string();
    }

    fn check_conversion_progress(&mut self) {
        let mut messages = Vec::new();

        if let Some(ref receiver) = self.conversion_receiver {
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }

        for message in messages {
            match message {
                ConversionMessage::Progress(progress) => {
                    self.progress = Some(progress);
                    self.status_message = format!(
                        "Converting... {:.1}%",
                        self.progress.as_ref().unwrap().percentage
                    );
                }
                ConversionMessage::Completed(output_path) => {
                    self.is_converting = false;
                    self.progress = None;
                    self.conversion_receiver = None;
                    self.status_message =
                        format!("Conversion completed: {}", output_path.display());
                }
                ConversionMessage::Error(error) => {
                    self.is_converting = false;
                    self.progress = None;
                    self.conversion_receiver = None;
                    self.error_message = Some(error.clone());
                    self.status_message = "Conversion failed".to_string();
                }
            }
        }
    }

    fn apply_preset(&mut self, preset_name: &str) {
        if let Some(preset) = self.preset_manager.get_preset(preset_name) {
            self.mode = preset.mode.clone();
            self.video_format = preset.video_format.clone();
            self.video_codec = preset.video_codec.clone();
            self.audio_codec = preset.audio_codec.clone();
            self.video_bitrate = preset.video_bitrate.clone().unwrap_or_default();
            self.audio_bitrate = preset.audio_bitrate.clone().unwrap_or_default();
            self.resolution = preset.resolution.clone().unwrap_or_default();
            self.frame_rate = preset.frame_rate.clone().unwrap_or_default();
            self.metadata_options = preset.metadata_options.clone();
            self.selected_preset = Some(preset_name.to_string());
            self.status_message = format!("Applied preset: {}", preset_name);
        }
    }

    fn save_current_preset(&mut self) {
        if !self.new_preset_name.is_empty() {
            let preset = ConversionPreset {
                name: self.new_preset_name.clone(),
                mode: self.mode.clone(),
                video_format: self.video_format.clone(),
                video_codec: self.video_codec.clone(),
                audio_codec: self.audio_codec.clone(),
                video_bitrate: if self.video_bitrate.is_empty() {
                    None
                } else {
                    Some(self.video_bitrate.clone())
                },
                audio_bitrate: if self.audio_bitrate.is_empty() {
                    None
                } else {
                    Some(self.audio_bitrate.clone())
                },
                resolution: if self.resolution.is_empty() {
                    None
                } else {
                    Some(self.resolution.clone())
                },
                frame_rate: if self.frame_rate.is_empty() {
                    None
                } else {
                    Some(self.frame_rate.clone())
                },
                metadata_options: self.metadata_options.clone(),
            };

            self.preset_manager.add_preset(preset);
            self.status_message = format!("Saved preset: {}", self.new_preset_name);
            self.new_preset_name.clear();
            self.show_save_preset = false;
        }
    }

    fn check_ffmpeg(&mut self) {
        let result = check_ffmpeg_installation();
        self.ffmpeg_status = Some(result);
    }

    fn check_for_updates(&mut self) {
        if let Some(updater) = self.updater.clone() {
            self.update_status = Some(UpdateStatus::CheckingForUpdates);

            self.runtime.spawn(async move {
                let status = updater.check_for_updates().await;
                println!("Update status: {:?}", status);
            });
        }
    }

    fn start_update_download(&mut self, update_info: UpdateInfo) {
        if let Some(updater) = self.updater.clone() {
            let (tx, rx) = mpsc::unbounded_channel();
            self.download_progress_receiver = Some(rx);
            self.download_progress = 0.0;
            self.update_status = Some(UpdateStatus::DownloadingUpdate(0.0));

            let runtime = Arc::clone(&self.runtime);
            runtime.spawn(async move {
                match updater.download_update(&update_info, Some(tx)).await {
                    Ok(file_path) => {
                        println!("Update downloaded to: {:?}", file_path);
                        // Auto-install the update
                        match updater.apply_update(&file_path).await {
                            Ok(()) => {
                                println!("Update applied successfully, restarting...");
                                let _ = updater.restart_application().await;
                            }
                            Err(e) => {
                                println!("Failed to apply update: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to download update: {}", e);
                    }
                }
            });
        }
    }

    fn check_download_progress(&mut self) {
        if let Some(ref mut receiver) = self.download_progress_receiver {
            while let Ok(progress) = receiver.try_recv() {
                self.download_progress = progress;
                self.update_status = Some(UpdateStatus::DownloadingUpdate(progress));

                if progress >= 100.0 {
                    self.update_status = Some(UpdateStatus::InstallingUpdate);
                }
            }
        }
    }
}

impl eframe::App for FFmpegApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Check for conversion progress updates
        self.check_conversion_progress();

        // Check for download progress updates
        self.check_download_progress();

        // Main UI
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("FFmpeg Rust");
            ui.separator();

            // File Selection Section
            ui.group(|ui| {
                ui.label(RichText::new("File Selection").strong());

                ui.horizontal(|ui| {
                    if ui.button("Select Input File").clicked() {
                        self.select_input_file();
                    }

                    if let Some(ref file) = self.input_file {
                        ui.label(format!(
                            "📁 {}",
                            file.file_name().unwrap_or_default().to_string_lossy()
                        ));
                    } else {
                        ui.label("No file selected");
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Select Output Folder").clicked() {
                        self.select_output_folder();
                    }

                    if let Some(ref folder) = self.output_folder {
                        ui.label(format!("📂 {}", folder.display()));
                    } else {
                        ui.label("No folder selected");
                    }
                });
            });

            ui.add_space(10.0);

            // Mode and Settings Section
            ui.group(|ui| {
                ui.label(RichText::new("Conversion Settings").strong());

                // Mode selection
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    ui.radio_value(&mut self.mode, ConversionMode::Convert, "Convert");
                    ui.radio_value(&mut self.mode, ConversionMode::Remux, "Remux");
                });

                ui.separator();

                // Format selection for both Convert and Remux modes
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    egui::ComboBox::from_id_source("video_format")
                        .selected_text(self.video_format.display_name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.video_format, VideoFormat::Mp4, "MP4");
                            ui.selectable_value(&mut self.video_format, VideoFormat::Mkv, "MKV");
                            ui.selectable_value(&mut self.video_format, VideoFormat::Mov, "MOV");
                            ui.selectable_value(&mut self.video_format, VideoFormat::Avi, "AVI");
                            ui.selectable_value(&mut self.video_format, VideoFormat::Webm, "WebM");
                        });
                });

                if self.mode == ConversionMode::Convert {
                    // Video codec
                    ui.horizontal(|ui| {
                        ui.label("Video:");
                        egui::ComboBox::from_id_source("video_codec")
                            .selected_text(self.video_codec.display_name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.video_codec,
                                    VideoCodec::H264,
                                    "H.264",
                                );
                                ui.selectable_value(
                                    &mut self.video_codec,
                                    VideoCodec::H265,
                                    "H.265",
                                );
                                ui.selectable_value(&mut self.video_codec, VideoCodec::VP9, "VP9");
                                ui.selectable_value(
                                    &mut self.video_codec,
                                    VideoCodec::Copy,
                                    "Copy",
                                );
                            });
                    });

                    // Audio codec
                    ui.horizontal(|ui| {
                        ui.label("Audio:");
                        egui::ComboBox::from_id_source("audio_codec")
                            .selected_text(self.audio_codec.display_name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.audio_codec, AudioCodec::Aac, "AAC");
                                ui.selectable_value(&mut self.audio_codec, AudioCodec::Mp3, "MP3");
                                ui.selectable_value(
                                    &mut self.audio_codec,
                                    AudioCodec::Flac,
                                    "FLAC",
                                );
                                ui.selectable_value(
                                    &mut self.audio_codec,
                                    AudioCodec::Pcm16,
                                    "PCM (16-bit)",
                                );
                                ui.selectable_value(
                                    &mut self.audio_codec,
                                    AudioCodec::Copy,
                                    "Copy",
                                );
                            });
                    });

                    // Optional settings
                    ui.collapsing("Advanced Settings", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Video Bitrate:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.video_bitrate)
                                    .id(egui::Id::new("video_bitrate_input")),
                            );
                            ui.label("(e.g., 2M, 1500k)");
                        });

                        ui.horizontal(|ui| {
                            ui.label("Audio Bitrate:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.audio_bitrate)
                                    .id(egui::Id::new("audio_bitrate_input")),
                            );
                            ui.label("(e.g., 128k, 320k)");
                        });

                        ui.horizontal(|ui| {
                            ui.label("Resolution:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.resolution)
                                    .id(egui::Id::new("resolution_input")),
                            );
                            ui.label("(e.g., 1920x1080)");
                        });

                        ui.horizontal(|ui| {
                            ui.label("Frame Rate:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.frame_rate)
                                    .id(egui::Id::new("frame_rate_input")),
                            );
                            ui.label("(e.g., 30, 60)");
                        });
                    });
                } else {
                    // Remux mode - show metadata options
                    ui.separator();

                    ui.collapsing("Metadata Options", |ui| {
                        // File-level metadata
                        ui.label(RichText::new("File-level metadata").strong());
                        ui.checkbox(
                            &mut self.metadata_options.copy_file_metadata,
                            "Copy file metadata (title, date, encoder, tags, etc.)",
                        );

                        ui.add_space(5.0);

                        // Stream-level metadata
                        ui.label(RichText::new("Stream-level metadata").strong());

                        ui.horizontal(|ui| {
                            ui.label("Video Language:");
                            egui::ComboBox::from_id_source("video_language")
                                .selected_text(
                                    MetadataOptions::get_common_languages()
                                        .iter()
                                        .find(|(code, _)| {
                                            *code == self.metadata_options.video_language
                                        })
                                        .map(|(_, name)| *name)
                                        .unwrap_or("Undetermined"),
                                )
                                .show_ui(ui, |ui| {
                                    for (code, name) in MetadataOptions::get_common_languages() {
                                        ui.selectable_value(
                                            &mut self.metadata_options.video_language,
                                            code.to_string(),
                                            name,
                                        );
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Audio Language:");
                            egui::ComboBox::from_id_source("audio_language")
                                .selected_text(
                                    MetadataOptions::get_common_languages()
                                        .iter()
                                        .find(|(code, _)| {
                                            *code == self.metadata_options.audio_language
                                        })
                                        .map(|(_, name)| *name)
                                        .unwrap_or("Undetermined"),
                                )
                                .show_ui(ui, |ui| {
                                    for (code, name) in MetadataOptions::get_common_languages() {
                                        ui.selectable_value(
                                            &mut self.metadata_options.audio_language,
                                            code.to_string(),
                                            name,
                                        );
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Video Title:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.metadata_options.video_title)
                                    .id(egui::Id::new("video_title_input")),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Audio Title:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.metadata_options.audio_title)
                                    .id(egui::Id::new("audio_title_input")),
                            );
                        });

                        ui.add_space(5.0);

                        // Chapters
                        ui.label(RichText::new("Chapters").strong());
                        ui.checkbox(&mut self.metadata_options.copy_chapters, "Copy chapters");

                        ui.add_space(5.0);

                        // Attachments (MKV only)
                        ui.label(RichText::new("Attachments (MKV only)").strong());
                        ui.checkbox(
                            &mut self.metadata_options.copy_attachments,
                            "Copy attachments (e.g., fonts, cover art)",
                        );
                    });
                }
            });

            ui.add_space(10.0);

            // Progress Section
            if self.is_converting || self.progress.is_some() {
                ui.group(|ui| {
                    ui.label(RichText::new("Progress").strong());

                    if let Some(ref progress) = self.progress {
                        // Progress bar
                        let progress_bar = egui::ProgressBar::new(progress.percentage / 100.0)
                            .text(format!("{:.1}%", progress.percentage));
                        ui.add(progress_bar);

                        // Time information
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Time: {} / {}",
                                progress.current_time, progress.total_time
                            ));

                            if let Some(remaining) = progress.time_remaining {
                                let remaining_secs = remaining.as_secs();
                                let hours = remaining_secs / 3600;
                                let minutes = (remaining_secs % 3600) / 60;
                                let seconds = remaining_secs % 60;

                                if hours > 0 {
                                    ui.label(format!(
                                        "Remaining: {:02}:{:02}:{:02}",
                                        hours, minutes, seconds
                                    ));
                                } else {
                                    ui.label(format!("Remaining: {:02}:{:02}", minutes, seconds));
                                }
                            }
                        });
                    }

                    if self.is_converting {
                        if ui.button("Stop Conversion").clicked() {
                            self.stop_conversion();
                        }
                    }
                });

                ui.add_space(10.0);
            }

            // Presets Section
            ui.group(|ui| {
                ui.label(RichText::new("Custom Presets").strong());

                ui.horizontal(|ui| {
                    let preset_names: Vec<String> = self
                        .preset_manager
                        .list_presets()
                        .iter()
                        .map(|p| p.name.clone())
                        .collect();

                    egui::ComboBox::from_id_source("preset_selector")
                        .selected_text(
                            self.selected_preset
                                .as_deref()
                                .unwrap_or("Select preset..."),
                        )
                        .show_ui(ui, |ui| {
                            for preset_name in preset_names {
                                if ui.selectable_label(false, &preset_name).clicked() {
                                    self.apply_preset(&preset_name);
                                }
                            }
                        });

                    if ui.button("Save Current").clicked() {
                        self.show_save_preset = true;
                    }

                    if let Some(preset_name) = self.selected_preset.clone() {
                        if ui.button("Delete").clicked() {
                            self.preset_manager.remove_preset(&preset_name);
                            self.selected_preset = None;
                            self.status_message = format!("Deleted preset: {}", preset_name);
                        }
                    }
                });

                if self.show_save_preset {
                    ui.horizontal(|ui| {
                        ui.label("Preset Name:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_preset_name)
                                .id(egui::Id::new("preset_name_input")),
                        );

                        if ui.button("Save").clicked() {
                            self.save_current_preset();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_save_preset = false;
                            self.new_preset_name.clear();
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Control buttons
            ui.horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    if ui
                        .add_enabled(
                            self.can_start_conversion(),
                            egui::Button::new("Start Conversion"),
                        )
                        .clicked()
                    {
                        self.start_conversion();
                    }

                    if ui.button("Help").clicked() {
                        self.show_help_dialog = true;
                    }
                });
            });

            ui.add_space(10.0);

            // Status and Error messages
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.status_message);
            });

            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                if ui.button("Clear Error").clicked() {
                    self.error_message = None;
                }
            }
        });

        // Help Dialog
        if self.show_help_dialog {
            egui::Window::new("Help")
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        if ui.button("Check FFmpeg").clicked() {
                            self.check_ffmpeg();
                        }

                        if let Some(ref status) = self.ffmpeg_status {
                            match status {
                                Ok(version) => {
                                    ui.colored_label(
                                        egui::Color32::GREEN,
                                        format!("FFmpeg found: {}", version),
                                    );
                                }
                                Err(error) => {
                                    ui.colored_label(
                                        egui::Color32::RED,
                                        format!("FFmpeg error: {}", error),
                                    );
                                }
                            }
                        }

                        ui.separator();

                        if ui.button("Check for Updates").clicked() {
                            self.check_for_updates();
                        }

                        if let Some(ref status) = self.update_status {
                            match status {
                                UpdateStatus::CheckingForUpdates => {
                                    ui.label("Checking for updates...");
                                }
                                UpdateStatus::UpdateAvailable(info) => {
                                    ui.colored_label(
                                        egui::Color32::GREEN,
                                        format!("Update available: v{}", info.version),
                                    );
                                    if ui.button("Download & Install").clicked() {
                                        if let Some(UpdateStatus::UpdateAvailable(ref info)) =
                                            self.update_status.clone()
                                        {
                                            self.start_update_download(info.clone());
                                        }
                                    }
                                }
                                UpdateStatus::NoUpdateAvailable => {
                                    ui.colored_label(
                                        egui::Color32::GREEN,
                                        "You have the latest version",
                                    );
                                }
                                UpdateStatus::DownloadingUpdate(progress) => {
                                    ui.label(format!("Downloading update: {:.1}%", progress));
                                    ui.add(egui::ProgressBar::new(progress / 100.0));
                                }
                                UpdateStatus::InstallingUpdate => {
                                    ui.colored_label(egui::Color32::YELLOW, "Installing update...");
                                }

                                UpdateStatus::Error(error) => {
                                    ui.colored_label(
                                        egui::Color32::RED,
                                        format!("Update failed: {}", error),
                                    );
                                }
                            }
                        }

                        ui.separator();

                        if ui.button("About").clicked() {
                            self.show_about_dialog = true;
                        }

                        ui.separator();

                        if ui.button("Close").clicked() {
                            self.show_help_dialog = false;
                        }
                    });
                });
        }

        // About Dialog
        if self.show_about_dialog {
            egui::Window::new("About")
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("FFmpeg Rust");
                        ui.label("Version 1.0.0");
                        ui.add_space(10.0);
                        ui.label("A simple, minimalistic GUI for video conversion and remuxing using FFmpeg");
                        ui.add_space(10.0);
                        ui.hyperlink_to("GitHub Repository", "https://github.com/yourusername/ffmpegrust");
                        ui.add_space(10.0);

                        if ui.button("Close").clicked() {
                            self.show_about_dialog = false;
                        }
                    });
                });
        }

        // Update Dialog
        if self.show_update_dialog {
            match self.update_status.clone() {
                Some(UpdateStatus::UpdateAvailable(info)) if self.show_update_dialog => {
                    egui::Window::new("Update Available")
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.label(format!("Version {} is available", info.version));
                                ui.add_space(5.0);

                                ui.label("Release Notes:");
                                ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                    ui.label(&info.release_notes);
                                });

                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    if ui.button("Download & Install").clicked() {
                                        self.start_update_download(info.clone());
                                        self.show_update_dialog = false;
                                    }

                                    if ui.button("Later").clicked() {
                                        self.show_update_dialog = false;
                                    }

                                    if ui.button("Skip This Version").clicked() {
                                        self.show_update_dialog = false;
                                        self.update_status = None;
                                    }
                                });
                            });
                        });
                }
                Some(UpdateStatus::DownloadingUpdate(progress)) => {
                    egui::Window::new("Downloading Update")
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.label("Downloading update...");
                                ui.add_space(10.0);

                                ui.add(egui::ProgressBar::new(progress / 100.0)
                                    .text(format!("{:.1}%", progress)));

                                ui.add_space(10.0);
                                ui.small("Please wait, the application will restart automatically after installation.");
                            });
                        });
                }
                Some(UpdateStatus::InstallingUpdate) => {
                    egui::Window::new("Installing Update")
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.label("Installing update...");
                                ui.add_space(10.0);
                                ui.spinner();
                                ui.add_space(10.0);
                                ui.small("Application will restart automatically.");
                            });
                        });
                }
                _ => {}
            }
        }

        // Request repaint for progress updates
        if self.is_converting {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save configuration on exit
        self.config.save();
    }
}
