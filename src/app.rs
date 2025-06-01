use crate::conversion::{ConversionTask, ConversionSettings, ConversionStatus, ConversionMode};
use crate::config::AppConfig;
use std::sync::mpsc;

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
    pub input_file: String,
    pub output_file: String,
    pub mode: ConversionMode,
    pub container: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub quality: String,
    pub is_converting: bool,
    pub progress: crate::conversion::ConversionProgress,
    pub error: Option<String>,
    pub status_receiver: Option<mpsc::Receiver<ConversionStatus>>,
    pub use_hardware_accel: bool,
    pub smart_copy: bool,
    pub conversion_task: Option<ConversionTask>,
    pub config: AppConfig,
    pub active_tab: ActiveTab,
}

impl Default for FFmpegApp {
    fn default() -> Self {
        Self::new()
    }
}

impl FFmpegApp {
    pub fn new() -> Self {
        let config = AppConfig::load();
        
        let mut app = Self {
            input_file: String::new(),
            output_file: String::new(),
            mode: ConversionMode::default(),
            container: config.default_container.clone(),
            video_codec: config.default_video_codec.clone(),
            audio_codec: config.default_audio_codec.clone(),
            quality: config.default_quality.clone(),
            is_converting: false,
            progress: Default::default(),
            error: None,
            status_receiver: None,
            use_hardware_accel: config.use_hardware_accel,
            smart_copy: config.smart_copy,
            conversion_task: None,
            config,
            active_tab: ActiveTab::default(),
        };

        app.check_ffmpeg();
        app
    }

    fn check_ffmpeg(&mut self) {
        match std::process::Command::new("ffmpeg").arg("-version").output() {
            Ok(output) => {
                if !output.status.success() {
                    self.error = Some("FFmpeg not working properly".to_string());
                }
            }
            Err(_) => {
                self.error = Some("FFmpeg not found in PATH. Please install FFmpeg.".to_string());
            }
        }
    }

    pub fn select_input(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Video", &["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ts"]);
        
        if let Some(ref dir) = self.config.last_input_dir {
            dialog = dialog.set_directory(dir);
        }
        
        if let Some(path) = dialog.pick_file() {
            self.input_file = path.to_string_lossy().to_string();
            self.config.update_last_input_dir(&self.input_file);
            self.auto_generate_output();
            self.error = None;
            self.save_config();
        }
    }

    pub fn select_output(&mut self) {
        let mut dialog = rfd::FileDialog::new();
        
        if let Some(ref dir) = self.config.last_output_dir {
            dialog = dialog.set_directory(dir);
        }
        
        if let Some(path) = dialog.save_file() {
            self.output_file = path.to_string_lossy().to_string();
            self.config.update_last_output_dir(&self.output_file);
            self.save_config();
        }
    }

    fn auto_generate_output(&mut self) {
        if let Some(path) = std::path::Path::new(&self.input_file).parent() {
            if let Some(stem) = std::path::Path::new(&self.input_file).file_stem() {
                let suffix = if self.mode == ConversionMode::Remux {
                    "_remux"
                } else {
                    "_converted"
                };
                let ext = &self.container;
                let output_path = path.join(format!("{}{}.{}", stem.to_string_lossy(), suffix, ext));
                self.output_file = output_path.to_string_lossy().to_string();
            }
        }
    }

    pub fn start_conversion(&mut self) {
        let settings = ConversionSettings {
            mode: self.mode.clone(),
            video_codec: self.video_codec.clone(),
            audio_codec: self.audio_codec.clone(),
            quality: self.quality.clone(),
            use_hardware_accel: self.use_hardware_accel,
            smart_copy: self.smart_copy,
            container: self.container.clone(),
        };

        let mut task = ConversionTask::new(
            self.input_file.clone(),
            self.output_file.clone(),
            settings,
        );

        match task.execute() {
            Ok(rx) => {
                self.status_receiver = Some(rx);
                self.is_converting = true;
                self.error = None;
                self.progress = Default::default();
                self.conversion_task = Some(task);
            }
            Err(e) => {
                self.error = Some(e.user_message());
            }
        }
    }

    pub fn stop_conversion(&mut self) {
        if let Some(task) = &self.conversion_task {
            task.cancel();
        }
        self.is_converting = false;
        self.status_receiver = None;
        self.conversion_task = None;
        self.progress = Default::default();
    }

    pub fn update_output_extension(&mut self) {
        if !self.output_file.is_empty() {
            let path = std::path::Path::new(&self.output_file);
            if let Some(stem) = path.file_stem() {
                if let Some(parent) = path.parent() {
                    let new_path = parent.join(format!("{}.{}", stem.to_string_lossy(), self.container));
                    self.output_file = new_path.to_string_lossy().to_string();
                }
            }
        }
    }

    pub fn update_status(&mut self) {
        let mut should_clear_receiver = false;
        let mut new_error = None;
        
        if let Some(rx) = &self.status_receiver {
            while let Ok(status) = rx.try_recv() {
                match status {
                    ConversionStatus::Starting => {
                        self.progress = Default::default();
                    }
                    ConversionStatus::InProgress(progress) => {
                        self.progress = progress;
                    }
                    ConversionStatus::Completed => {
                        self.is_converting = false;
                        should_clear_receiver = true;
                        self.progress.percentage = 100.0;
                        self.conversion_task = None;
                    }
                    ConversionStatus::Failed(error) => {
                        self.is_converting = false;
                        should_clear_receiver = true;
                        new_error = Some(error);
                        self.conversion_task = None;
                    }
                    ConversionStatus::Cancelled => {
                        self.is_converting = false;
                        should_clear_receiver = true;
                        self.conversion_task = None;
                    }
                }
            }
        }
        
        if should_clear_receiver {
            self.status_receiver = None;
        }
        
        if let Some(error) = new_error {
            self.error = Some(error);
        }
    }

    pub fn apply_preset(&mut self, preset_name: &str) {
        match preset_name {
            "Web (H.264/MP4)" => {
                self.mode = ConversionMode::Convert;
                self.container = "mp4".to_string();
                self.video_codec = "libx264".to_string();
                self.audio_codec = "aac".to_string();
                self.quality = "23".to_string();
            }
            "High Quality (H.265/MKV)" => {
                self.mode = ConversionMode::Convert;
                self.container = "mkv".to_string();
                self.video_codec = "libx265".to_string();
                self.audio_codec = "aac".to_string();
                self.quality = "20".to_string();
            }
            "Small File (H.265)" => {
                self.mode = ConversionMode::Convert;
                self.video_codec = "libx265".to_string();
                self.audio_codec = "aac".to_string();
                self.quality = "28".to_string();
            }
            "Fast Remux" => {
                self.mode = ConversionMode::Remux;
            }
            "MOV PCM (Pro Audio)" => {
                self.mode = ConversionMode::Convert;
                self.container = "mov".to_string();
                self.smart_copy = true;
                self.use_hardware_accel = true;
            }
            _ => {}
        }
        self.update_output_extension();
        self.save_config();
    }

    fn save_config(&self) {
        self.config.save();
    }

    pub fn update_config_from_current_settings(&mut self) {
        self.config.default_video_codec = self.video_codec.clone();
        self.config.default_audio_codec = self.audio_codec.clone();
        self.config.default_quality = self.quality.clone();
        self.config.default_container = self.container.clone();
        self.config.use_hardware_accel = self.use_hardware_accel;
        self.config.smart_copy = self.smart_copy;
    }
}