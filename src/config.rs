use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub last_input_folder: Option<PathBuf>,
    pub last_output_folder: Option<PathBuf>,
    pub auto_check_updates: bool,
    pub window_width: f32,
    pub window_height: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_input_folder: None,
            last_output_folder: None,
            auto_check_updates: true,
            window_width: 1000.0,
            window_height: 600.0,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("ffmpegrust").join("config.json");

            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str::<Config>(&content) {
                        return config;
                    }
                }
            }
        }

        Self::default()
    }

    pub fn save(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("ffmpegrust");

            if let Ok(()) = std::fs::create_dir_all(&app_config_dir) {
                let config_path = app_config_dir.join("config.json");

                if let Ok(content) = serde_json::to_string_pretty(self) {
                    let _ = std::fs::write(&config_path, content);
                }
            }
        }
    }

    pub fn update_input_folder(&mut self, path: Option<PathBuf>) {
        self.last_input_folder = path;
        self.save();
    }

    pub fn update_output_folder(&mut self, path: Option<PathBuf>) {
        self.last_output_folder = path;
        self.save();
    }
}
