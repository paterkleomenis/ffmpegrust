use crate::conversion::{ConversionMode, ConversionSettings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PresetError {
    #[error("Preset not found: {name}")]
    NotFound { name: String },
    #[error("Invalid preset data: {message}")]
    InvalidData { message: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionPreset {
    pub name: String,
    pub description: String,
    pub settings: ConversionSettings,
    pub is_builtin: bool,
    pub category: PresetCategory,
    pub tags: Vec<String>,
    pub created_at: Option<std::time::SystemTime>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PresetCategory {
    Web,
    Archive,
    Streaming,
    Mobile,
    Professional,
    Custom,
}

impl std::fmt::Display for PresetCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Web => write!(f, "Web Optimized"),
            Self::Archive => write!(f, "Archive Quality"),
            Self::Streaming => write!(f, "Streaming"),
            Self::Mobile => write!(f, "Mobile Devices"),
            Self::Professional => write!(f, "Professional"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

pub struct PresetManager {
    presets: HashMap<String, ConversionPreset>,
    custom_presets_path: Option<PathBuf>,
}

impl PresetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            presets: HashMap::new(),
            custom_presets_path: Self::get_custom_presets_path(),
        };
        manager.load_builtin_presets();
        manager
    }

    fn get_custom_presets_path() -> Option<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            Some(config_dir.join("ffmpegrust").join("presets.json"))
        } else {
            None
        }
    }

    fn load_builtin_presets(&mut self) {
        let builtin_presets = vec![
            ConversionPreset {
                name: "Web Standard (H.264/MP4)".to_string(),
                description: "Standard web video with good quality/size balance".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libx264".to_string(),
                    audio_codec: "aac".to_string(),
                    quality: "23".to_string(),
                    use_hardware_accel: true,
                    container: "mp4".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Web,
                tags: vec![
                    "h264".to_string(),
                    "web".to_string(),
                    "standard".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "High Quality (H.265/MKV)".to_string(),
                description: "High quality video with modern H.265 codec".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libx265".to_string(),
                    audio_codec: "aac".to_string(),
                    quality: "20".to_string(),
                    use_hardware_accel: true,
                    container: "mkv".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Archive,
                tags: vec![
                    "h265".to_string(),
                    "quality".to_string(),
                    "hevc".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Small File Size (H.265)".to_string(),
                description: "Optimized for smallest file size while maintaining watchable quality"
                    .to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libx265".to_string(),
                    audio_codec: "aac".to_string(),
                    quality: "28".to_string(),
                    use_hardware_accel: true,
                    container: "mp4".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Mobile,
                tags: vec![
                    "small".to_string(),
                    "mobile".to_string(),
                    "h265".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Fast Remux to MP4".to_string(),
                description: "Fast remux to MP4 container without re-encoding video/audio"
                    .to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Remux,
                    video_codec: "copy".to_string(),
                    audio_codec: "copy".to_string(),
                    quality: "".to_string(),
                    use_hardware_accel: false,
                    container: "mp4".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Web,
                tags: vec!["fast".to_string(), "remux".to_string(), "mp4".to_string()],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Fast Remux to MOV".to_string(),
                description: "Fast remux to MOV container without re-encoding video/audio"
                    .to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Remux,
                    video_codec: "copy".to_string(),
                    audio_codec: "copy".to_string(),
                    quality: "".to_string(),
                    use_hardware_accel: false,
                    container: "mov".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Web,
                tags: vec!["fast".to_string(), "remux".to_string(), "mov".to_string()],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Fast Remux to MKV".to_string(),
                description: "Fast remux to MKV container without re-encoding video/audio"
                    .to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Remux,
                    video_codec: "copy".to_string(),
                    audio_codec: "copy".to_string(),
                    quality: "".to_string(),
                    use_hardware_accel: false,
                    container: "mkv".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Web,
                tags: vec!["fast".to_string(), "remux".to_string(), "mkv".to_string()],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Fast Remux to WebM".to_string(),
                description: "Fast remux to WebM container without re-encoding video/audio"
                    .to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Remux,
                    video_codec: "copy".to_string(),
                    audio_codec: "copy".to_string(),
                    quality: "".to_string(),
                    use_hardware_accel: false,
                    container: "webm".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Web,
                tags: vec!["fast".to_string(), "remux".to_string(), "webm".to_string()],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Professional PCM Archive".to_string(),
                description:
                    "High-quality uncompressed PCM audio for archival and professional use"
                        .to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libx264".to_string(),
                    audio_codec: "pcm_s24le".to_string(),
                    quality: "18".to_string(),
                    use_hardware_accel: false,
                    container: "mov".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Professional,
                tags: vec![
                    "pcm".to_string(),
                    "uncompressed".to_string(),
                    "archive".to_string(),
                    "professional".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Audio-Only PCM WAV".to_string(),
                description: "Extract audio to uncompressed PCM WAV format".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "copy".to_string(),
                    audio_codec: "pcm_s16le".to_string(),
                    quality: "".to_string(),
                    use_hardware_accel: false,
                    container: "wav".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Professional,
                tags: vec![
                    "audio".to_string(),
                    "pcm".to_string(),
                    "wav".to_string(),
                    "extract".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "YouTube Upload".to_string(),
                description: "Optimized for YouTube uploads with recommended settings".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libx264".to_string(),
                    audio_codec: "aac".to_string(),
                    quality: "21".to_string(),
                    use_hardware_accel: true,
                    container: "mp4".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Streaming,
                tags: vec![
                    "youtube".to_string(),
                    "upload".to_string(),
                    "streaming".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Twitch Stream Archive".to_string(),
                description: "Settings for archiving Twitch streams".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libx264".to_string(),
                    audio_codec: "aac".to_string(),
                    quality: "22".to_string(),
                    use_hardware_accel: true,
                    container: "mkv".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Streaming,
                tags: vec![
                    "twitch".to_string(),
                    "stream".to_string(),
                    "archive".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Professional Archive (ProRes)".to_string(),
                description: "Professional quality with ProRes codec for editing".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "prores_ks".to_string(),
                    audio_codec: "pcm_s24le".to_string(),
                    quality: "3".to_string(),
                    use_hardware_accel: false,
                    container: "mov".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Professional,
                tags: vec![
                    "prores".to_string(),
                    "professional".to_string(),
                    "editing".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "Mobile Device (H.264/Low)".to_string(),
                description: "Optimized for mobile devices and slow connections".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libx264".to_string(),
                    audio_codec: "aac".to_string(),
                    quality: "26".to_string(),
                    use_hardware_accel: true,
                    container: "mp4".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Mobile,
                tags: vec![
                    "mobile".to_string(),
                    "phone".to_string(),
                    "low-bandwidth".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "AV1 Future-Proof".to_string(),
                description: "Next-generation AV1 codec for maximum efficiency".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libaom-av1".to_string(),
                    audio_codec: "libopus".to_string(),
                    quality: "30".to_string(),
                    use_hardware_accel: false,
                    container: "mkv".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Archive,
                tags: vec![
                    "av1".to_string(),
                    "future".to_string(),
                    "efficiency".to_string(),
                ],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
            ConversionPreset {
                name: "WebM for Web".to_string(),
                description: "WebM format optimized for web playback".to_string(),
                settings: ConversionSettings {
                    mode: ConversionMode::Convert,
                    video_codec: "libvpx-vp9".to_string(),
                    audio_codec: "libopus".to_string(),
                    quality: "24".to_string(),
                    use_hardware_accel: false,
                    container: "webm".to_string(),
                },
                is_builtin: true,
                category: PresetCategory::Web,
                tags: vec!["webm".to_string(), "vp9".to_string(), "web".to_string()],
                created_at: None,
                author: Some("FFmpeg Rust".to_string()),
            },
        ];

        for preset in builtin_presets {
            self.presets.insert(preset.name.clone(), preset);
        }
    }

    pub async fn load_custom_presets(&mut self) -> Result<(), PresetError> {
        if let Some(presets_path) = &self.custom_presets_path {
            if presets_path.exists() {
                let content = tokio::fs::read_to_string(presets_path).await?;
                let custom_presets: Vec<ConversionPreset> = serde_json::from_str(&content)?;

                for preset in custom_presets {
                    if !preset.is_builtin {
                        self.presets.insert(preset.name.clone(), preset);
                    }
                }

                tracing::info!("Loaded custom presets from {:?}", presets_path);
            }
        }
        Ok(())
    }

    pub async fn save_custom_presets(&self) -> Result<(), PresetError> {
        if let Some(presets_path) = &self.custom_presets_path {
            // Create directory if it doesn't exist
            if let Some(parent) = presets_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // Collect only custom presets
            let custom_presets: Vec<&ConversionPreset> = self
                .presets
                .values()
                .filter(|preset| !preset.is_builtin)
                .collect();

            let content = serde_json::to_string_pretty(&custom_presets)?;
            tokio::fs::write(presets_path, content).await?;

            tracing::info!(
                "Saved {} custom presets to {:?}",
                custom_presets.len(),
                presets_path
            );
        }
        Ok(())
    }

    pub fn get_preset(&self, name: &str) -> Option<&ConversionPreset> {
        self.presets.get(name)
    }

    pub fn get_all_presets(&self) -> Vec<&ConversionPreset> {
        self.presets.values().collect()
    }

    pub fn get_presets_by_category(&self, category: &PresetCategory) -> Vec<&ConversionPreset> {
        self.presets
            .values()
            .filter(|preset| preset.category == *category)
            .collect()
    }

    pub fn search_presets(&self, query: &str) -> Vec<&ConversionPreset> {
        let query_lower = query.to_lowercase();
        self.presets
            .values()
            .filter(|preset| {
                preset.name.to_lowercase().contains(&query_lower)
                    || preset.description.to_lowercase().contains(&query_lower)
                    || preset
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn get_builtin_presets(&self) -> Vec<&ConversionPreset> {
        self.presets
            .values()
            .filter(|preset| preset.is_builtin)
            .collect()
    }

    pub fn get_custom_presets(&self) -> Vec<&ConversionPreset> {
        self.presets
            .values()
            .filter(|preset| !preset.is_builtin)
            .collect()
    }

    pub async fn add_custom_preset(
        &mut self,
        mut preset: ConversionPreset,
    ) -> Result<(), PresetError> {
        // Ensure it's marked as custom
        preset.is_builtin = false;
        preset.category = PresetCategory::Custom;
        preset.created_at = Some(std::time::SystemTime::now());

        // Validate preset name doesn't conflict
        if self.presets.contains_key(&preset.name) {
            return Err(PresetError::InvalidData {
                message: format!("Preset '{}' already exists", preset.name),
            });
        }

        // Validate settings
        self.validate_preset_settings(&preset.settings)?;

        self.presets.insert(preset.name.clone(), preset);
        self.save_custom_presets().await?;

        Ok(())
    }

    pub async fn update_custom_preset(
        &mut self,
        name: &str,
        mut preset: ConversionPreset,
    ) -> Result<(), PresetError> {
        // Check if preset exists and is custom
        if let Some(existing) = self.presets.get(name) {
            if existing.is_builtin {
                return Err(PresetError::InvalidData {
                    message: "Cannot modify builtin presets".to_string(),
                });
            }
        } else {
            return Err(PresetError::NotFound {
                name: name.to_string(),
            });
        }

        // Validate settings
        self.validate_preset_settings(&preset.settings)?;

        preset.is_builtin = false;
        preset.category = PresetCategory::Custom;

        self.presets.insert(name.to_string(), preset);
        self.save_custom_presets().await?;

        Ok(())
    }

    pub async fn remove_custom_preset(&mut self, name: &str) -> Result<(), PresetError> {
        if let Some(preset) = self.presets.get(name) {
            if preset.is_builtin {
                return Err(PresetError::InvalidData {
                    message: "Cannot remove builtin presets".to_string(),
                });
            }
        } else {
            return Err(PresetError::NotFound {
                name: name.to_string(),
            });
        }

        self.presets.remove(name);
        self.save_custom_presets().await?;

        Ok(())
    }

    pub fn create_preset_from_current_settings(
        &self,
        name: String,
        description: String,
        settings: ConversionSettings,
    ) -> ConversionPreset {
        ConversionPreset {
            name,
            description,
            settings,
            is_builtin: false,
            category: PresetCategory::Custom,
            tags: vec!["user-created".to_string()],
            created_at: Some(std::time::SystemTime::now()),
            author: Some("User".to_string()),
        }
    }

    pub async fn export_presets(&self, path: &std::path::Path) -> Result<(), PresetError> {
        let all_presets: Vec<&ConversionPreset> = self.presets.values().collect();
        let content = serde_json::to_string_pretty(&all_presets)?;
        tokio::fs::write(path, content).await?;

        tracing::info!("Exported {} presets to {:?}", all_presets.len(), path);
        Ok(())
    }

    pub async fn import_presets(
        &mut self,
        path: &std::path::Path,
        replace_existing: bool,
    ) -> Result<usize, PresetError> {
        let content = tokio::fs::read_to_string(path).await?;
        let imported_presets: Vec<ConversionPreset> = serde_json::from_str(&content)?;

        let mut imported_count = 0;

        for mut preset in imported_presets {
            // Mark as custom when importing
            preset.is_builtin = false;
            preset.created_at = Some(std::time::SystemTime::now());

            // Validate settings
            if let Err(e) = self.validate_preset_settings(&preset.settings) {
                tracing::warn!("Skipping invalid preset '{}': {}", preset.name, e);
                continue;
            }

            // Check for conflicts
            if self.presets.contains_key(&preset.name) && !replace_existing {
                tracing::warn!("Skipping existing preset '{}'", preset.name);
                continue;
            }

            self.presets.insert(preset.name.clone(), preset);
            imported_count += 1;
        }

        if imported_count > 0 {
            self.save_custom_presets().await?;
        }

        tracing::info!("Imported {} presets from {:?}", imported_count, path);
        Ok(imported_count)
    }

    fn validate_preset_settings(&self, settings: &ConversionSettings) -> Result<(), PresetError> {
        // Basic validation
        if settings.video_codec.is_empty() {
            return Err(PresetError::InvalidData {
                message: "Video codec cannot be empty".to_string(),
            });
        }

        if settings.audio_codec.is_empty() {
            return Err(PresetError::InvalidData {
                message: "Audio codec cannot be empty".to_string(),
            });
        }

        if settings.container.is_empty() {
            return Err(PresetError::InvalidData {
                message: "Container format cannot be empty".to_string(),
            });
        }

        // Quality validation
        if !settings.quality.is_empty() {
            if let Ok(quality) = settings.quality.parse::<u32>() {
                if quality > 51 {
                    return Err(PresetError::InvalidData {
                        message: "CRF quality value cannot exceed 51".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn get_preset_categories(&self) -> Vec<PresetCategory> {
        vec![
            PresetCategory::Web,
            PresetCategory::Archive,
            PresetCategory::Streaming,
            PresetCategory::Mobile,
            PresetCategory::Professional,
            PresetCategory::Custom,
        ]
    }

    pub fn get_category_description(&self, category: &PresetCategory) -> &'static str {
        match category {
            PresetCategory::Web => "Optimized for web playback and sharing",
            PresetCategory::Archive => "High quality settings for long-term storage",
            PresetCategory::Streaming => "Optimized for streaming platforms",
            PresetCategory::Mobile => "Optimized for mobile devices and bandwidth",
            PresetCategory::Professional => "Professional and editing-friendly formats",
            PresetCategory::Custom => "User-created custom presets",
        }
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PresetManager {
    fn clone(&self) -> Self {
        let mut cloned = Self::new();
        cloned.presets = self.presets.clone();
        cloned.custom_presets_path = self.custom_presets_path.clone();
        cloned
    }
}
