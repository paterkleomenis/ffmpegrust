use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionPreset {
    pub name: String,
    pub mode: ConversionMode,
    pub video_format: VideoFormat,
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
    pub video_bitrate: Option<String>,
    pub audio_bitrate: Option<String>,
    pub resolution: Option<String>,
    pub frame_rate: Option<String>,
    pub metadata_options: MetadataOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataOptions {
    pub copy_file_metadata: bool,
    pub copy_chapters: bool,
    pub copy_attachments: bool,
    pub video_language: String,
    pub audio_language: String,
    pub subtitle_language: String,
    pub video_title: String,
    pub audio_title: String,
    pub subtitle_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversionMode {
    Convert,
    Remux,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoFormat {
    Mp4,
    Mkv,
    Mov,
    Avi,
    Webm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoCodec {
    H264,
    H265,
    VP9,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AudioCodec {
    Aac,
    Mp3,
    Flac,
    Pcm16,
    Copy,
}

impl VideoFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::Mkv => "mkv",
            VideoFormat::Mov => "mov",
            VideoFormat::Avi => "avi",
            VideoFormat::Webm => "webm",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "MP4",
            VideoFormat::Mkv => "MKV",
            VideoFormat::Mov => "MOV",
            VideoFormat::Avi => "AVI",
            VideoFormat::Webm => "WebM",
        }
    }
}

impl VideoCodec {
    pub fn display_name(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "H.264",
            VideoCodec::H265 => "H.265",
            VideoCodec::VP9 => "VP9",
            VideoCodec::Copy => "Copy",
        }
    }

    pub fn ffmpeg_name(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "libx264",
            VideoCodec::H265 => "libx265",
            VideoCodec::VP9 => "libvpx-vp9",
            VideoCodec::Copy => "copy",
        }
    }
}

impl AudioCodec {
    pub fn display_name(&self) -> &'static str {
        match self {
            AudioCodec::Aac => "AAC",
            AudioCodec::Mp3 => "MP3",
            AudioCodec::Flac => "FLAC",
            AudioCodec::Pcm16 => "PCM (16-bit)",
            AudioCodec::Copy => "Copy",
        }
    }

    pub fn ffmpeg_name(&self) -> &'static str {
        match self {
            AudioCodec::Aac => "aac",
            AudioCodec::Mp3 => "libmp3lame",
            AudioCodec::Flac => "flac",
            AudioCodec::Pcm16 => "pcm_s16le",
            AudioCodec::Copy => "copy",
        }
    }
}

#[derive(Debug, Default)]
pub struct PresetManager {
    presets: HashMap<String, ConversionPreset>,
}

impl PresetManager {
    pub fn new() -> Self {
        let mut manager = Self::default();
        manager.load_presets();
        manager
    }

    pub fn load_presets(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let presets_path = config_dir.join("ffmpegrust").join("presets.json");

            if presets_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&presets_path) {
                    if let Ok(presets) =
                        serde_json::from_str::<HashMap<String, ConversionPreset>>(&content)
                    {
                        self.presets = presets;
                    }
                }
            }
        }
    }

    pub fn save_presets(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("ffmpegrust");

            if let Ok(()) = std::fs::create_dir_all(&app_config_dir) {
                let presets_path = app_config_dir.join("presets.json");

                if let Ok(content) = serde_json::to_string_pretty(&self.presets) {
                    let _ = std::fs::write(&presets_path, content);
                }
            }
        }
    }

    pub fn add_preset(&mut self, preset: ConversionPreset) {
        self.presets.insert(preset.name.clone(), preset);
        self.save_presets();
    }

    pub fn remove_preset(&mut self, name: &str) {
        self.presets.remove(name);
        self.save_presets();
    }

    pub fn get_preset(&self, name: &str) -> Option<&ConversionPreset> {
        self.presets.get(name)
    }

    pub fn list_presets(&self) -> Vec<&ConversionPreset> {
        self.presets.values().collect()
    }
}

impl Default for ConversionPreset {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            mode: ConversionMode::Convert,
            video_format: VideoFormat::Mp4,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
            video_bitrate: None,
            audio_bitrate: None,
            resolution: None,
            frame_rate: None,
            metadata_options: MetadataOptions::default(),
        }
    }
}

impl Default for MetadataOptions {
    fn default() -> Self {
        Self {
            copy_file_metadata: true,
            copy_chapters: true,
            copy_attachments: true,
            video_language: "und".to_string(),
            audio_language: "und".to_string(),
            subtitle_language: "und".to_string(),
            video_title: String::new(),
            audio_title: String::new(),
            subtitle_title: String::new(),
        }
    }
}

impl MetadataOptions {
    pub fn get_common_languages() -> Vec<(&'static str, &'static str)> {
        vec![
            ("und", "Undetermined"),
            ("eng", "English"),
            ("spa", "Spanish"),
            ("fra", "French"),
            ("deu", "German"),
            ("ita", "Italian"),
            ("por", "Portuguese"),
            ("rus", "Russian"),
            ("jpn", "Japanese"),
            ("kor", "Korean"),
            ("chi", "Chinese"),
            ("ara", "Arabic"),
            ("hin", "Hindi"),
            ("nld", "Dutch"),
            ("swe", "Swedish"),
            ("nor", "Norwegian"),
            ("dan", "Danish"),
            ("fin", "Finnish"),
            ("pol", "Polish"),
            ("cze", "Czech"),
            ("hun", "Hungarian"),
            ("tur", "Turkish"),
            ("gre", "Greek"),
            ("heb", "Hebrew"),
            ("tha", "Thai"),
            ("vie", "Vietnamese"),
        ]
    }
}
