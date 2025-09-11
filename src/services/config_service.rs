use crate::config::AppConfig;
use crate::events::{AppEvent, EventSender};
use crate::services::Service;
use std::path::PathBuf;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load config: {0}")]
    LoadFailed(String),
    #[error("Failed to save config: {0}")]
    SaveFailed(String),
    #[error("Invalid config path: {path}")]
    InvalidPath { path: String },
    #[error("Permission denied for config file: {path}")]
    PermissionDenied { path: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct ConfigService {
    config: std::sync::Arc<RwLock<AppConfig>>,
    event_sender: EventSender,
    config_path: Option<PathBuf>,
}

impl ConfigService {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            config: std::sync::Arc::new(RwLock::new(AppConfig::default())),
            event_sender,
            config_path: Self::get_config_path(),
        }
    }

    fn get_config_path() -> Option<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            Some(config_dir.join("ffmpegrust").join("config.json"))
        } else {
            None
        }
    }

    pub async fn load_config(&self) -> Result<AppConfig, ConfigError> {
        let config = if let Some(config_path) = &self.config_path {
            if config_path.exists() {
                match tokio::fs::read_to_string(&config_path).await {
                    Ok(config_data) => match serde_json::from_str(&config_data) {
                        Ok(loaded_config) => {
                            tracing::info!("Config loaded from: {:?}", config_path);
                            loaded_config
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse config file, using defaults: {}", e);
                            AppConfig::default()
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to read config file, using defaults: {}", e);
                        AppConfig::default()
                    }
                }
            } else {
                tracing::info!("Config file doesn't exist, using defaults");
                AppConfig::default()
            }
        } else {
            tracing::warn!("Could not determine config directory, using defaults");
            AppConfig::default()
        };

        // Update the internal config
        {
            let mut internal_config = self.config.write().await;
            *internal_config = config.clone();
        }

        // Send event
        if let Err(e) = self.event_sender.send(AppEvent::ConfigLoaded) {
            tracing::error!("Failed to send config loaded event: {}", e);
        }

        Ok(config)
    }

    pub async fn save_config(&self) -> Result<(), ConfigError> {
        let config = {
            let config_guard = self.config.read().await;
            config_guard.clone()
        };

        if let Some(config_path) = &self.config_path {
            // Ensure config directory exists
            if let Some(parent) = config_path.parent() {
                if !parent.exists() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| {
                        ConfigError::SaveFailed(format!("Failed to create config directory: {}", e))
                    })?;
                }
            }

            // Save config
            let config_data = serde_json::to_string_pretty(&config)?;
            tokio::fs::write(&config_path, config_data)
                .await
                .map_err(|e| {
                    ConfigError::SaveFailed(format!("Failed to write config file: {}", e))
                })?;

            tracing::info!("Config saved to: {:?}", config_path);

            // Send event
            if let Err(e) = self.event_sender.send(AppEvent::ConfigSaved) {
                tracing::error!("Failed to send config saved event: {}", e);
            }

            Ok(())
        } else {
            Err(ConfigError::InvalidPath {
                path: "Could not determine config path".to_string(),
            })
        }
    }

    pub async fn get_config(&self) -> AppConfig {
        let config_guard = self.config.read().await;
        config_guard.clone()
    }

    pub async fn update_config<F>(&self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut AppConfig),
    {
        {
            let mut config_guard = self.config.write().await;
            updater(&mut config_guard);
        }

        // Send event
        if let Err(e) = self.event_sender.send(AppEvent::SettingsChanged) {
            tracing::error!("Failed to send settings changed event: {}", e);
        }

        Ok(())
    }

    pub async fn update_last_input_dir(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        self.update_config(|config| {
            if let Some(parent) = path.parent() {
                config.last_input_dir = Some(parent.to_string_lossy().to_string());
            }
        })
        .await
    }

    pub async fn update_last_output_dir(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        self.update_config(|config| {
            if let Some(parent) = path.parent() {
                config.last_output_dir = Some(parent.to_string_lossy().to_string());
            }
        })
        .await
    }

    pub async fn update_default_codecs(
        &self,
        video_codec: String,
        audio_codec: String,
    ) -> Result<(), ConfigError> {
        self.update_config(|config| {
            config.default_video_codec = video_codec;
            config.default_audio_codec = audio_codec;
        })
        .await
    }

    pub async fn update_default_quality(&self, quality: String) -> Result<(), ConfigError> {
        self.update_config(|config| {
            config.default_quality = quality;
        })
        .await
    }

    pub async fn update_default_container(&self, container: String) -> Result<(), ConfigError> {
        self.update_config(|config| {
            config.default_container = container;
        })
        .await
    }

    pub async fn update_hardware_acceleration(&self, enabled: bool) -> Result<(), ConfigError> {
        self.update_config(|config| {
            config.use_hardware_accel = enabled;
        })
        .await
    }

    pub async fn update_smart_copy(&self, _enabled: bool) -> Result<(), ConfigError> {
        self.update_config(|_config| {
            // Smart copy is now automatic - no config needed
        })
        .await
    }

    pub async fn update_window_size(&self, width: f32, height: f32) -> Result<(), ConfigError> {
        self.update_config(|config| {
            config.window_width = width;
            config.window_height = height;
        })
        .await
    }

    pub async fn reset_to_defaults(&self) -> Result<(), ConfigError> {
        {
            let mut config_guard = self.config.write().await;
            *config_guard = AppConfig::default();
        }

        // Send event
        if let Err(e) = self.event_sender.send(AppEvent::SettingsChanged) {
            tracing::error!("Failed to send settings changed event: {}", e);
        }

        Ok(())
    }

    pub async fn get_last_input_dir(&self) -> Option<PathBuf> {
        let config = self.config.read().await;
        config.last_input_dir.as_ref().map(|s| PathBuf::from(s))
    }

    pub async fn get_last_output_dir(&self) -> Option<PathBuf> {
        let config = self.config.read().await;
        config.last_output_dir.as_ref().map(|s| PathBuf::from(s))
    }

    pub async fn export_config(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        let config = {
            let config_guard = self.config.read().await;
            config_guard.clone()
        };

        let config_data = serde_json::to_string_pretty(&config)?;
        tokio::fs::write(path, config_data)
            .await
            .map_err(|e| ConfigError::SaveFailed(format!("Failed to export config: {}", e)))?;

        tracing::info!("Config exported to: {:?}", path);
        Ok(())
    }

    pub async fn import_config(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        let config_data = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::LoadFailed(format!("Failed to read import file: {}", e)))?;

        let imported_config: AppConfig = serde_json::from_str(&config_data)?;

        {
            let mut config_guard = self.config.write().await;
            *config_guard = imported_config;
        }

        // Send event
        if let Err(e) = self.event_sender.send(AppEvent::SettingsChanged) {
            tracing::error!("Failed to send settings changed event: {}", e);
        }

        tracing::info!("Config imported from: {:?}", path);
        Ok(())
    }

    pub async fn backup_config(&self) -> Result<PathBuf, ConfigError> {
        if let Some(config_path) = &self.config_path {
            if config_path.exists() {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let backup_path =
                    config_path.with_file_name(format!("config_backup_{}.json", timestamp));

                tokio::fs::copy(config_path, &backup_path)
                    .await
                    .map_err(|e| {
                        ConfigError::SaveFailed(format!("Failed to create backup: {}", e))
                    })?;

                tracing::info!("Config backed up to: {:?}", backup_path);
                Ok(backup_path)
            } else {
                Err(ConfigError::LoadFailed(
                    "Config file doesn't exist to backup".to_string(),
                ))
            }
        } else {
            Err(ConfigError::InvalidPath {
                path: "Could not determine config path for backup".to_string(),
            })
        }
    }
}

#[async_trait::async_trait]
impl Service for ConfigService {
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_config().await?;
        tracing::info!("Config service initialized");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_config().await?;
        tracing::info!("Config service shutdown");
        Ok(())
    }
}
