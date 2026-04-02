use crate::config::Config;
use crate::conversion::{
    ConversionMessage, ConversionProgress, ConversionTask, check_ffmpeg_installation,
    generate_output_filename,
};
use crate::presets::{
    AudioCodec, ConversionMode, ConversionPreset, MetadataOptions, PresetManager, VideoCodec,
    VideoFormat,
};
use crate::updater::{UpdateInfo, UpdateStatus, Updater};
use egui::{
    CentralPanel, Color32, Context, RichText, ScrollArea, SidePanel, Stroke, TopBottomPanel,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct FFmpegApp {
    config: Config,
    runtime: Arc<Runtime>,

    input_file: Option<PathBuf>,
    output_folder: Option<PathBuf>,
    output_file_name: String,

    mode: ConversionMode,
    video_format: VideoFormat,
    video_codec: VideoCodec,
    audio_codec: AudioCodec,

    video_bitrate: String,
    audio_bitrate: String,
    resolution: String,
    frame_rate: String,

    metadata_options: MetadataOptions,

    is_converting: bool,
    progress: Option<ConversionProgress>,
    conversion_receiver: Option<Receiver<ConversionMessage>>,
    status_message: String,
    error_message: Option<String>,

    preset_manager: PresetManager,
    selected_preset: Option<String>,
    new_preset_name: String,
    show_save_preset: bool,

    show_help_dialog: bool,
    show_about_dialog: bool,
    ffmpeg_status: Option<Result<String, String>>,

    updater: Option<Updater>,
    update_status: Option<UpdateStatus>,
    update_status_receiver: Option<Receiver<UpdateStatus>>,
    download_progress_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<f32>>,

    style_initialized: bool,
}

impl Default for FFmpegApp {
    fn default() -> Self {
        Self {
            config: Config::load(),
            runtime: Arc::new(tokio::runtime::Runtime::new().expect("runtime init failed")),

            input_file: None,
            output_folder: None,
            output_file_name: String::new(),

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
            update_status_receiver: None,
            download_progress_receiver: None,

            style_initialized: false,
        }
    }
}

impl FFmpegApp {
    fn bg_base() -> Color32 {
        Color32::from_rgb(6, 6, 8)
    }

    fn bg_panel() -> Color32 {
        Color32::from_rgb(10, 10, 13)
    }

    fn bg_card() -> Color32 {
        Color32::from_rgb(14, 14, 18)
    }

    fn border_soft() -> Color32 {
        Color32::from_rgb(45, 45, 54)
    }

    fn accent() -> Color32 {
        Color32::from_rgb(162, 162, 176)
    }

    fn text_main() -> Color32 {
        Color32::from_rgb(235, 235, 239)
    }

    fn success() -> Color32 {
        Color32::from_rgb(104, 184, 149)
    }

    fn danger() -> Color32 {
        Color32::from_rgb(225, 106, 106)
    }

    pub fn new(runtime: Arc<Runtime>) -> Self {
        let mut app = Self {
            runtime,
            ..Default::default()
        };

        if let Ok(updater) = Updater::new("1.0.0", "pater/ffmpegrust") {
            app.updater = Some(updater);
        }

        app
    }

    fn initialize_style(&mut self, ctx: &Context) {
        if self.style_initialized {
            return;
        }

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Self::bg_panel();
        visuals.extreme_bg_color = Self::bg_base();
        visuals.faint_bg_color = Self::bg_card();
        visuals.window_fill = Self::bg_panel();
        visuals.override_text_color = Some(Self::text_main());
        visuals.selection.bg_fill = Color32::from_rgb(60, 60, 72);
        visuals.selection.stroke = Stroke::new(1.0, Self::accent());

        visuals.widgets.noninteractive.bg_fill = Self::bg_card();
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::border_soft());
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(20, 20, 25);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::border_soft());
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 30, 38);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::accent());
        visuals.widgets.active.bg_fill = Color32::from_rgb(36, 36, 46);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Self::accent());
        visuals.window_stroke = Stroke::new(1.0, Self::border_soft());
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.interact_size.y = 24.0;
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(12.5));
        style
            .text_styles
            .insert(egui::TextStyle::Small, egui::FontId::proportional(11.0));
        ctx.set_style(style);

        self.style_initialized = true;
    }

    fn select_input_file(&mut self) {
        let default_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let start_dir = self.config.last_input_folder.clone().unwrap_or(default_dir);

        if let Some(file) = rfd::FileDialog::new()
            .set_title("Select Input Video File")
            .add_filter(
                "Video Files",
                &[
                    "mp4", "mkv", "mov", "avi", "webm", "flv", "wmv", "m4v", "3gp", "ts", "mts",
                    "m2ts", "vob", "mpg", "mpeg", "ogv",
                ],
            )
            .set_directory(start_dir)
            .pick_file()
        {
            if let Some(parent) = file.parent() {
                self.config.update_input_folder(Some(parent.to_path_buf()));
                if self.output_folder.is_none() {
                    let out = parent.to_path_buf();
                    self.output_folder = Some(out.clone());
                    self.config.update_output_folder(Some(out));
                }
            }

            let default_output = generate_output_filename(&file, &self.video_format);
            self.output_file_name = default_output
                .file_stem()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            self.input_file = Some(file);
            self.error_message = None;
            self.status_message = "Input file selected".to_string();
        }
    }

    fn select_output_folder(&mut self) {
        let default_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let start_dir = self
            .config
            .last_output_folder
            .clone()
            .unwrap_or(default_dir);

        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Select Output Folder")
            .set_directory(start_dir)
            .pick_folder()
        {
            self.config.update_output_folder(Some(folder.clone()));
            self.output_folder = Some(folder);
            self.status_message = "Output folder selected".to_string();
        }
    }

    fn select_output_file(&mut self) {
        let default_dir = self
            .output_folder
            .clone()
            .or_else(|| self.config.last_output_folder.clone())
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let default_stem = if self.output_file_name.trim().is_empty() {
            self.input_file
                .as_ref()
                .map(|f| generate_output_filename(f, &self.video_format))
                .and_then(|p| p.file_stem().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "output".to_string())
        } else {
            Self::normalize_output_name(&self.output_file_name)
        };

        let default_name = format!("{default_stem}.{}", self.video_format.extension());

        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Output File")
            .set_directory(default_dir)
            .set_file_name(&default_name)
            .save_file()
        {
            if let Some(parent) = path.parent() {
                let parent_buf = parent.to_path_buf();
                self.output_folder = Some(parent_buf.clone());
                self.config.update_output_folder(Some(parent_buf));
            }

            if let Some(name) = path.file_stem() {
                self.output_file_name = name.to_string_lossy().to_string();
            }

            self.status_message = "Output file selected".to_string();
        }
    }

    fn can_start_conversion(&self) -> bool {
        !self.is_converting
    }

    fn build_current_preset(&self, name: String) -> ConversionPreset {
        ConversionPreset {
            name,
            mode: self.mode.clone(),
            video_format: self.video_format.clone(),
            video_codec: self.video_codec.clone(),
            audio_codec: self.audio_codec.clone(),
            video_bitrate: (!self.video_bitrate.is_empty()).then(|| self.video_bitrate.clone()),
            audio_bitrate: (!self.audio_bitrate.is_empty()).then(|| self.audio_bitrate.clone()),
            resolution: (!self.resolution.is_empty()).then(|| self.resolution.clone()),
            frame_rate: (!self.frame_rate.is_empty()).then(|| self.frame_rate.clone()),
            metadata_options: self.metadata_options.clone(),
        }
    }

    fn start_conversion(&mut self) {
        let Some(input_file) = self.input_file.clone() else {
            self.error_message = Some("Please select an input file".to_string());
            self.status_message = "Input file required".to_string();
            return;
        };

        let output_folder = self
            .output_folder
            .clone()
            .or_else(|| input_file.parent().map(|p| p.to_path_buf()))
            .or_else(|| self.config.last_output_folder.clone())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        if self.output_folder.is_none() {
            self.output_folder = Some(output_folder.clone());
            self.config
                .update_output_folder(Some(output_folder.clone()));
            self.status_message = "Using input folder as output folder".to_string();
        }

        if self.is_converting {
            return;
        }

        let normalized_name = Self::normalize_output_name(&self.output_file_name);

        let output_file = if normalized_name.is_empty() {
            let output_filename = generate_output_filename(&input_file, &self.video_format);
            let Some(file_name) = output_filename.file_name() else {
                self.error_message = Some("Failed to generate output filename".to_string());
                return;
            };
            output_folder.join(file_name)
        } else {
            self.output_file_name = normalized_name.clone();
            output_folder.join(format!(
                "{normalized_name}.{}",
                self.video_format.extension()
            ))
        };
        let preset = self.build_current_preset("Current".to_string());

        let (sender, receiver) = std::sync::mpsc::channel();
        self.conversion_receiver = Some(receiver);

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

        if let Some(receiver) = &self.conversion_receiver {
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }

        for message in messages {
            match message {
                ConversionMessage::Progress(progress) => {
                    let percentage = progress.percentage;
                    self.progress = Some(progress);
                    self.status_message = format!("Converting... {percentage:.1}%");
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
                    self.error_message = Some(error);
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
            self.status_message = format!("Applied preset: {preset_name}");
        }
    }

    fn save_current_preset(&mut self) {
        if self.new_preset_name.trim().is_empty() {
            return;
        }

        let preset_name = self.new_preset_name.trim().to_string();
        let preset = self.build_current_preset(preset_name.clone());
        self.preset_manager.add_preset(preset);

        self.status_message = format!("Saved preset: {preset_name}");
        self.selected_preset = Some(preset_name);
        self.new_preset_name.clear();
        self.show_save_preset = false;
    }

    fn check_ffmpeg(&mut self) {
        self.ffmpeg_status = Some(check_ffmpeg_installation());
    }

    fn check_for_updates(&mut self) {
        let Some(updater) = self.updater.clone() else {
            self.update_status = Some(UpdateStatus::Error("Updater unavailable".to_string()));
            return;
        };

        let (sender, receiver): (Sender<UpdateStatus>, Receiver<UpdateStatus>) =
            std::sync::mpsc::channel();
        self.update_status_receiver = Some(receiver);
        self.update_status = Some(UpdateStatus::CheckingForUpdates);

        self.runtime.spawn(async move {
            let status = updater.check_for_updates().await;
            let _ = sender.send(status);
        });
    }

    fn start_update_download(&mut self, update_info: UpdateInfo) {
        let Some(updater) = self.updater.clone() else {
            self.update_status = Some(UpdateStatus::Error("Updater unavailable".to_string()));
            return;
        };

        let (progress_tx, progress_rx) = mpsc::unbounded_channel();
        self.download_progress_receiver = Some(progress_rx);

        let (status_tx, status_rx): (Sender<UpdateStatus>, Receiver<UpdateStatus>) =
            std::sync::mpsc::channel();
        self.update_status_receiver = Some(status_rx);
        self.update_status = Some(UpdateStatus::DownloadingUpdate(0.0));

        self.runtime.spawn(async move {
            match updater
                .download_update(&update_info, Some(progress_tx))
                .await
            {
                Ok(file_path) => {
                    let _ = status_tx.send(UpdateStatus::InstallingUpdate);
                    match updater.apply_update(&file_path).await {
                        Ok(()) => {
                            let _ = updater.restart_application().await;
                        }
                        Err(err) => {
                            let _ = status_tx.send(UpdateStatus::Error(format!(
                                "Failed to apply update: {err}"
                            )));
                        }
                    }
                }
                Err(err) => {
                    let _ = status_tx.send(UpdateStatus::Error(format!(
                        "Failed to download update: {err}"
                    )));
                }
            }
        });
    }

    fn poll_async_updates(&mut self) {
        if let Some(receiver) = &self.update_status_receiver {
            while let Ok(status) = receiver.try_recv() {
                self.update_status = Some(status);
            }
        }

        if let Some(receiver) = &mut self.download_progress_receiver {
            while let Ok(progress) = receiver.try_recv() {
                self.update_status = Some(UpdateStatus::DownloadingUpdate(progress));
            }
        }
    }

    fn short_path(path: &Path) -> String {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| path.display().to_string())
    }

    fn ellipsize(text: &str, max_chars: usize) -> String {
        let count = text.chars().count();
        if count <= max_chars {
            return text.to_string();
        }

        if max_chars <= 3 {
            return "...".to_string();
        }

        let take = max_chars - 3;
        let mut out = String::new();
        for ch in text.chars().take(take) {
            out.push(ch);
        }
        out.push_str("...");
        out
    }

    fn normalize_output_name(name: &str) -> String {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let file_name = Path::new(trimmed)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(trimmed);

        Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name)
            .to_string()
    }

    fn output_filename_for_container(&self) -> String {
        let stem = Self::normalize_output_name(&self.output_file_name);
        if stem.is_empty() {
            format!("output.{}", self.video_format.extension())
        } else {
            format!("{stem}.{}", self.video_format.extension())
        }
    }

    fn section_card(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::group(ui.style())
            .fill(Self::bg_card())
            .stroke(Stroke::new(1.0, Self::border_soft()))
            .inner_margin(egui::Margin::symmetric(6, 6))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(title)
                        .strong()
                        .size(13.0)
                        .color(Self::text_main()),
                );
                ui.add_space(2.0);
                add_contents(ui);
            });
    }

    fn render_media_panel(&mut self, ui: &mut egui::Ui) {
        Self::section_card(ui, "Media", |ui| {
            egui::Grid::new("media_grid")
                .num_columns(3)
                .spacing(egui::vec2(6.0, 6.0))
                .show(ui, |ui| {
                    ui.label("Input");
                    let input = self
                        .input_file
                        .as_deref()
                        .map(Self::short_path)
                        .unwrap_or_else(|| "none".to_string());
                    ui.label(Self::ellipsize(&input, 42)).on_hover_text(&input);
                    if ui.button("Browse").clicked() {
                        self.select_input_file();
                    }
                    ui.end_row();

                    ui.label("Output");
                    let output = self
                        .output_folder
                        .as_deref()
                        .map(Self::short_path)
                        .unwrap_or_else(|| "none".to_string());
                    ui.label(Self::ellipsize(&output, 42))
                        .on_hover_text(&output);
                    if ui.button("Browse").clicked() {
                        self.select_output_folder();
                    }
                    ui.end_row();

                    ui.label("Name");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.output_file_name).desired_width(220.0),
                    );
                    if response.changed() {
                        self.output_file_name = Self::normalize_output_name(&self.output_file_name);
                    }
                    if ui.button("Save As").clicked() {
                        self.select_output_file();
                    }
                    ui.end_row();
                });

            ui.horizontal(|ui| {
                ui.label("Mode");
                ui.selectable_value(&mut self.mode, ConversionMode::Convert, "Convert");
                ui.selectable_value(&mut self.mode, ConversionMode::Remux, "Remux");
            });
        });
    }

    fn render_presets_panel(&mut self, ui: &mut egui::Ui) {
        Self::section_card(ui, "Presets", |ui| {
            let mut preset_names: Vec<String> = self
                .preset_manager
                .list_presets()
                .iter()
                .map(|preset| preset.name.clone())
                .collect();
            preset_names.sort_unstable();

            egui::ComboBox::from_id_salt("preset_selector")
                .selected_text(self.selected_preset.as_deref().unwrap_or("Choose preset"))
                .show_ui(ui, |ui| {
                    for preset_name in &preset_names {
                        if ui.selectable_label(false, preset_name).clicked() {
                            self.apply_preset(preset_name);
                        }
                    }
                });

            ui.horizontal_wrapped(|ui| {
                if ui.button("Save Current").clicked() {
                    self.show_save_preset = true;
                }

                if let Some(preset_name) = self.selected_preset.clone()
                    && ui.button("Delete").clicked()
                {
                    self.preset_manager.remove_preset(&preset_name);
                    self.selected_preset = None;
                    self.status_message = format!("Deleted preset: {preset_name}");
                }
            });

            if self.show_save_preset {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.new_preset_name);
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
    }

    fn render_conversion_panel(&mut self, ui: &mut egui::Ui) {
        Self::section_card(ui, "Conversion", |ui| {
            egui::Grid::new("conversion_grid")
                .num_columns(2)
                .spacing(egui::vec2(8.0, 6.0))
                .show(ui, |ui| {
                    ui.label("Container");
                    egui::ComboBox::from_id_salt("video_format")
                        .selected_text(self.video_format.display_name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.video_format, VideoFormat::Mp4, "MP4");
                            ui.selectable_value(&mut self.video_format, VideoFormat::Mkv, "MKV");
                            ui.selectable_value(&mut self.video_format, VideoFormat::Mov, "MOV");
                            ui.selectable_value(&mut self.video_format, VideoFormat::Avi, "AVI");
                            ui.selectable_value(&mut self.video_format, VideoFormat::Webm, "WebM");
                        });
                    ui.end_row();

                    if self.mode == ConversionMode::Convert {
                        ui.label("Video codec");
                        egui::ComboBox::from_id_salt("video_codec")
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
                        ui.end_row();

                        ui.label("Audio codec");
                        egui::ComboBox::from_id_salt("audio_codec")
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
                        ui.end_row();
                    }
                });

            if self.mode == ConversionMode::Convert {
                ui.collapsing("Advanced", |ui| {
                    egui::Grid::new("advanced_grid")
                        .num_columns(2)
                        .spacing(egui::vec2(8.0, 6.0))
                        .show(ui, |ui| {
                            ui.label("Video bitrate");
                            ui.text_edit_singleline(&mut self.video_bitrate);
                            ui.end_row();

                            ui.label("Audio bitrate");
                            ui.text_edit_singleline(&mut self.audio_bitrate);
                            ui.end_row();

                            ui.label("Resolution");
                            ui.text_edit_singleline(&mut self.resolution);
                            ui.end_row();

                            ui.label("Frame rate");
                            ui.text_edit_singleline(&mut self.frame_rate);
                            ui.end_row();
                        });
                });
            } else {
                ui.label("Metadata");
                ui.checkbox(
                    &mut self.metadata_options.copy_file_metadata,
                    "Copy file-level metadata",
                );
                ui.checkbox(&mut self.metadata_options.copy_chapters, "Copy chapters");
                ui.checkbox(
                    &mut self.metadata_options.copy_attachments,
                    "Copy attachments (MKV)",
                );

                ui.horizontal(|ui| {
                    ui.label("Video language");
                    egui::ComboBox::from_id_salt("video_language")
                        .selected_text(
                            MetadataOptions::get_common_languages()
                                .iter()
                                .find(|(code, _)| *code == self.metadata_options.video_language)
                                .map_or("Undetermined", |(_, name)| *name),
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
                    ui.label("Audio language");
                    egui::ComboBox::from_id_salt("audio_language")
                        .selected_text(
                            MetadataOptions::get_common_languages()
                                .iter()
                                .find(|(code, _)| *code == self.metadata_options.audio_language)
                                .map_or("Undetermined", |(_, name)| *name),
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
                    ui.label("Video title");
                    ui.text_edit_singleline(&mut self.metadata_options.video_title);
                });

                ui.horizontal(|ui| {
                    ui.label("Audio title");
                    ui.text_edit_singleline(&mut self.metadata_options.audio_title);
                });
            }
        });
    }

    fn render_progress_panel(&mut self, ui: &mut egui::Ui) {
        Self::section_card(ui, "Progress", |ui| {
            if let Some(progress) = &self.progress {
                ui.add(
                    egui::ProgressBar::new(progress.percentage / 100.0)
                        .desired_width(ui.available_width())
                        .text(format!("{:.1}%", progress.percentage)),
                );

                let mut info = format!("{} / {}", progress.current_time, progress.total_time);
                if let Some(remaining) = progress.time_remaining {
                    let remaining_secs = remaining.as_secs();
                    let minutes = remaining_secs / 60;
                    let seconds = remaining_secs % 60;
                    info.push_str(&format!(" | ETA {minutes:02}:{seconds:02}"));
                }
                ui.label(info);
            } else {
                let output_file_preview = if self.output_file_name.trim().is_empty() {
                    self.input_file
                        .as_ref()
                        .map(|f| generate_output_filename(f, &self.video_format))
                        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                        .unwrap_or_else(|| format!("output.{}", self.video_format.extension()))
                } else {
                    self.output_filename_for_container()
                };

                let input = self
                    .input_file
                    .as_deref()
                    .map(Self::short_path)
                    .unwrap_or_else(|| "none".to_string());
                let output = self
                    .output_folder
                    .as_deref()
                    .map(Self::short_path)
                    .unwrap_or_else(|| "none".to_string());
                ui.label(format!("Input: {}", Self::ellipsize(&input, 80)));
                ui.label(format!("Output: {}", Self::ellipsize(&output, 80)));
                ui.label(format!(
                    "File: {}",
                    Self::ellipsize(&output_file_preview, 80)
                ));
                ui.label("Ready");
            }

            if self.is_converting && ui.button("Stop Conversion").clicked() {
                self.stop_conversion();
            }
        });
    }

    fn render_actions_panel(&mut self, ui: &mut egui::Ui) {
        Self::section_card(ui, "Actions", |ui| {
            let start_label = if self.is_converting {
                "Converting..."
            } else {
                "Start Conversion"
            };

            let button = egui::Button::new(RichText::new(start_label).strong())
                .min_size(egui::vec2(170.0, 26.0))
                .fill(Color32::from_rgb(30, 30, 38))
                .stroke(Stroke::new(1.0, Self::accent()));

            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(self.can_start_conversion(), button)
                    .clicked()
                {
                    self.start_conversion();
                }

                if ui.button("Help").clicked() {
                    self.show_help_dialog = true;
                }
                if ui.button("About").clicked() {
                    self.show_about_dialog = true;
                }
                if ui.button("Check Updates").clicked() {
                    self.check_for_updates();
                }
                if self.is_converting && ui.button("Stop").clicked() {
                    self.stop_conversion();
                }
            });
        });
    }

    fn render_status_bar(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Status").strong());
                ui.label(&self.status_message);
                if let Some(error) = &self.error_message {
                    ui.colored_label(Self::danger(), format!("Error: {error}"));
                    if ui.button("Clear").clicked() {
                        self.error_message = None;
                    }
                }
            });
        });
    }

    fn render_help_dialog(&mut self, ctx: &Context) {
        if !self.show_help_dialog {
            return;
        }

        egui::Window::new("Help and Diagnostics")
            .resizable(false)
            .collapsible(false)
            .default_width(500.0)
            .show(ctx, |ui| {
                if ui.button("Check FFmpeg").clicked() {
                    self.check_ffmpeg();
                }

                if let Some(status) = &self.ffmpeg_status {
                    match status {
                        Ok(version) => {
                            ui.colored_label(Self::success(), version);
                        }
                        Err(err) => {
                            ui.colored_label(Self::danger(), err);
                        }
                    }
                }

                ui.separator();

                if ui.button("Check for Updates").clicked() {
                    self.check_for_updates();
                }

                if let Some(status) = self.update_status.clone() {
                    match status {
                        UpdateStatus::CheckingForUpdates => {
                            ui.label("Checking for updates...");
                        }
                        UpdateStatus::UpdateAvailable(info) => {
                            ui.colored_label(
                                Self::success(),
                                format!("Update available: v{}", info.version),
                            );
                            if ui.button("Download and Install").clicked() {
                                self.start_update_download(info.clone());
                            }

                            ui.add_space(6.0);
                            ui.label(RichText::new("Release Notes").strong());
                            ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                                ui.label(info.release_notes.clone());
                            });
                        }
                        UpdateStatus::NoUpdateAvailable => {
                            ui.label("You already have the latest version.");
                        }
                        UpdateStatus::DownloadingUpdate(progress) => {
                            ui.label(format!("Downloading update: {progress:.1}%"));
                            ui.add(egui::ProgressBar::new(progress / 100.0));
                        }
                        UpdateStatus::InstallingUpdate => {
                            ui.label("Installing update...");
                            ui.spinner();
                        }
                        UpdateStatus::Error(error) => {
                            ui.colored_label(Self::danger(), error);
                        }
                    }
                }

                ui.separator();
                if ui.button("Close").clicked() {
                    self.show_help_dialog = false;
                }
            });
    }

    fn render_about_dialog(&mut self, ctx: &Context) {
        if !self.show_about_dialog {
            return;
        }

        egui::Window::new("About FFmpeg Rust")
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("FFmpeg Rust");
                    ui.label("Version 1.0.0");
                    ui.add_space(8.0);
                    ui.label(
                        "A focused desktop tool for conversion and remux workflows powered by FFmpeg.",
                    );
                    ui.add_space(8.0);
                    ui.hyperlink_to(
                        "Project Repository",
                        "https://github.com/paterkleomenis/ffmpegrust",
                    );
                    ui.add_space(8.0);

                    if ui.button("Close").clicked() {
                        self.show_about_dialog = false;
                    }
                });
            });
    }
}

impl eframe::App for FFmpegApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.initialize_style(ctx);
        self.check_conversion_progress();
        self.poll_async_updates();

        TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("FFmpeg Rust").strong());
                let mode_label = match self.mode {
                    ConversionMode::Convert => "Mode: Convert",
                    ConversionMode::Remux => "Mode: Remux",
                };
                let state_label = if self.is_converting {
                    "State: Running"
                } else {
                    "State: Idle"
                };

                ui.label(RichText::new(mode_label).color(Self::accent()).strong());
                ui.label(RichText::new(state_label).color(Self::accent()).italics());
            });
        });

        self.render_status_bar(ctx);

        SidePanel::left("left_panel")
            .resizable(true)
            .default_width(360.0)
            .width_range(280.0..=460.0)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.render_media_panel(ui);
                    ui.add_space(6.0);
                    self.render_presets_panel(ui);
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.render_conversion_panel(ui);
                    ui.add_space(6.0);
                    self.render_progress_panel(ui);
                    ui.add_space(6.0);
                    self.render_actions_panel(ui);
                });
        });

        self.render_help_dialog(ctx);
        self.render_about_dialog(ctx);

        if self.is_converting
            || matches!(self.update_status, Some(UpdateStatus::DownloadingUpdate(_)))
        {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.config.save();
    }
}
