use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub last_input_dir: Option<String>,
    pub last_output_dir: Option<String>,
    pub default_video_codec: String,
    pub default_audio_codec: String,
    pub default_quality: String,
    pub default_container: String,
    pub use_hardware_accel: bool,
    pub window_width: f32,
    pub window_height: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_input_dir: None,
            last_output_dir: None,
            default_video_codec: "libx264".to_string(),
            default_audio_codec: "aac".to_string(),
            default_quality: "23".to_string(),
            default_container: "mp4".to_string(),
            use_hardware_accel: true,
            window_width: 700.0,
            window_height: 500.0,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                if let Ok(config_data) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str(&config_data) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(config_path) = Self::config_file_path() {
            if let Some(parent) = config_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(config_data) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(&config_path, config_data);
            }
        }
    }

    fn config_file_path() -> Option<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            Some(config_dir.join("ffmpegrust").join("config.json"))
        } else {
            None
        }
    }

    pub fn update_last_input_dir(&mut self, path: &str) {
        if let Some(parent) = std::path::Path::new(path).parent() {
            self.last_input_dir = Some(parent.to_string_lossy().to_string());
        }
    }

    pub fn update_last_output_dir(&mut self, path: &str) {
        if let Some(parent) = std::path::Path::new(path).parent() {
            self.last_output_dir = Some(parent.to_string_lossy().to_string());
        }
    }
}
