use crate::constants::{AUDIO_CODECS, CONTAINER_FORMATS, DEFAULT_CODECS, VIDEO_EXTENSIONS};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Input file is required")]
    MissingInputFile,
    #[error("Output file is required")]
    MissingOutputFile,
    #[error("Input file does not exist: {path}")]
    InputFileNotFound { path: String },
    #[error("Output directory does not exist: {path}")]
    OutputDirectoryNotFound { path: String },
    #[error("Insufficient permissions for output directory: {path}")]
    OutputPermissionDenied { path: String },
    #[error("Unsupported input file format: {format}")]
    UnsupportedInputFormat { format: String },
    #[error("Invalid codec: {codec}")]
    InvalidCodec { codec: String },
    #[error("Invalid quality value: {quality}")]
    InvalidQuality { quality: String },
    #[error("Invalid container format: {format}")]
    InvalidContainer { format: String },
    #[error("Input and output files cannot be the same")]
    SameInputOutput,
    #[error("Insufficient disk space for conversion")]
    InsufficientDiskSpace,
    #[error("FFmpeg not found or not accessible")]
    FFmpegNotAvailable,
    #[error("Custom validation error: {message}")]
    Custom { message: String },
}

#[derive(Clone)]
pub struct ValidationService;

impl ValidationService {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_input_file_sync(&self, path: &Path) -> Result<(), ValidationError> {
        if !path.exists() {
            return Err(ValidationError::InputFileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        if !path.is_file() {
            return Err(ValidationError::InputFileNotFound {
                path: format!("{} is not a file", path.display()),
            });
        }

        // Validate file extension
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                if !VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
                    return Err(ValidationError::UnsupportedInputFormat {
                        format: ext_str.to_string(),
                    });
                }
            }
        } else {
            return Err(ValidationError::UnsupportedInputFormat {
                format: "No file extension".to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_input_file(&self, path: Option<&Path>) -> Result<(), ValidationError> {
        let path = path.ok_or(ValidationError::MissingInputFile)?;

        if !path.exists() {
            return Err(ValidationError::InputFileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        if !path.is_file() {
            return Err(ValidationError::InputFileNotFound {
                path: format!("{} is not a file", path.display()),
            });
        }

        // Validate file extension
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                if !VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
                    return Err(ValidationError::UnsupportedInputFormat {
                        format: ext_str.to_string(),
                    });
                }
            }
        } else {
            return Err(ValidationError::UnsupportedInputFormat {
                format: "No file extension".to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_output_file_sync(&self, path: &Path) -> Result<(), ValidationError> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(ValidationError::OutputDirectoryNotFound {
                    path: parent.to_string_lossy().to_string(),
                });
            }

            // Check write permissions by attempting to create a test file
            let test_file = parent.join(".write_test_temp");
            match std::fs::write(&test_file, b"test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                }
                Err(_) => {
                    return Err(ValidationError::OutputPermissionDenied {
                        path: parent.to_string_lossy().to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn validate_output_file(&self, path: Option<&Path>) -> Result<(), ValidationError> {
        let path = path.ok_or(ValidationError::MissingOutputFile)?;

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(ValidationError::OutputDirectoryNotFound {
                    path: parent.to_string_lossy().to_string(),
                });
            }

            // Check write permissions by attempting to create a test file
            let test_file = parent.join(".write_test_temp");
            match std::fs::write(&test_file, b"test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                }
                Err(_) => {
                    return Err(ValidationError::OutputPermissionDenied {
                        path: parent.to_string_lossy().to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn validate_paths(
        &self,
        input: Option<&Path>,
        output: Option<&Path>,
    ) -> Result<(), ValidationError> {
        self.validate_input_file(input)?;
        self.validate_output_file(output)?;

        // Check if input and output are the same
        if let (Some(input_path), Some(output_path)) = (input, output) {
            if let (Ok(input_canonical), Ok(output_canonical)) =
                (input_path.canonicalize(), output_path.canonicalize())
            {
                if input_canonical == output_canonical {
                    return Err(ValidationError::SameInputOutput);
                }
            }
        }

        Ok(())
    }

    pub fn validate_video_codec(&self, codec: &str) -> Result<(), ValidationError> {
        if codec.is_empty() {
            return Err(ValidationError::InvalidCodec {
                codec: "Empty codec".to_string(),
            });
        }

        let valid_codecs: Vec<&str> = DEFAULT_CODECS.iter().map(|(codec, _)| *codec).collect();

        if !valid_codecs.contains(&codec) {
            return Err(ValidationError::InvalidCodec {
                codec: codec.to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_audio_codec(&self, codec: &str) -> Result<(), ValidationError> {
        if codec.is_empty() {
            return Err(ValidationError::InvalidCodec {
                codec: "Empty audio codec".to_string(),
            });
        }

        let valid_codecs: Vec<&str> = AUDIO_CODECS.iter().map(|(codec, _)| *codec).collect();

        if !valid_codecs.contains(&codec) {
            return Err(ValidationError::InvalidCodec {
                codec: codec.to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_quality(&self, quality: &str) -> Result<(), ValidationError> {
        if quality.is_empty() {
            return Ok(()); // Quality is optional for some codecs
        }

        // Try to parse as integer (CRF value)
        if let Ok(crf) = quality.parse::<u32>() {
            if crf > 51 {
                return Err(ValidationError::InvalidQuality {
                    quality: format!("CRF value {} is too high (max 51)", crf),
                });
            }
            return Ok(());
        }

        // Try to parse as bitrate (e.g., "1000k", "2M")
        if quality.ends_with('k') || quality.ends_with('K') {
            let number_part = &quality[..quality.len() - 1];
            if number_part.parse::<u32>().is_ok() {
                return Ok(());
            }
        }

        if quality.ends_with('M') || quality.ends_with('m') {
            let number_part = &quality[..quality.len() - 1];
            if number_part.parse::<f32>().is_ok() {
                return Ok(());
            }
        }

        Err(ValidationError::InvalidQuality {
            quality: quality.to_string(),
        })
    }

    pub fn validate_container(&self, container: &str) -> Result<(), ValidationError> {
        if container.is_empty() {
            return Err(ValidationError::InvalidContainer {
                format: "Empty container format".to_string(),
            });
        }

        let valid_containers: Vec<&str> = CONTAINER_FORMATS
            .iter()
            .map(|(format, _)| *format)
            .collect();

        if !valid_containers.contains(&container) {
            return Err(ValidationError::InvalidContainer {
                format: container.to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_conversion_settings(
        &self,
        video_codec: &str,
        audio_codec: &str,
        quality: &str,
        container: &str,
    ) -> Result<(), ValidationError> {
        self.validate_video_codec(video_codec)?;
        self.validate_audio_codec(audio_codec)?;
        self.validate_quality(quality)?;
        self.validate_container(container)?;

        // Additional cross-validation rules
        self.validate_codec_container_compatibility(video_codec, audio_codec, container)?;

        Ok(())
    }

    pub fn validate_codec_container_compatibility(
        &self,
        video_codec: &str,
        audio_codec: &str,
        container: &str,
    ) -> Result<(), ValidationError> {
        // Check for known incompatible combinations
        match (video_codec, container) {
            ("libvpx-vp9", "mp4") => {
                return Err(ValidationError::Custom {
                    message: "VP9 codec is not commonly supported in MP4 containers. Consider using WebM.".to_string(),
                });
            }
            ("libaom-av1", format) if format != "mkv" && format != "webm" => {
                return Err(ValidationError::Custom {
                    message: "AV1 codec is best supported in MKV or WebM containers.".to_string(),
                });
            }
            _ => {}
        }

        match (audio_codec, container) {
            ("libopus", "mp4") => {
                return Err(ValidationError::Custom {
                    message: "Opus audio is not supported in MP4 containers. Consider using AAC or another container.".to_string(),
                });
            }
            ("flac", "mp4") => {
                return Err(ValidationError::Custom {
                    message:
                        "FLAC audio in MP4 may have limited compatibility. Consider using AAC."
                            .to_string(),
                });
            }
            _ => {}
        }

        Ok(())
    }

    pub fn validate_ffmpeg_available(&self) -> Result<(), ValidationError> {
        match std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(ValidationError::FFmpegNotAvailable)
                }
            }
            Err(_) => Err(ValidationError::FFmpegNotAvailable),
        }
    }

    pub fn validate_ffprobe_available(&self) -> Result<(), ValidationError> {
        match std::process::Command::new("ffprobe")
            .arg("-version")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(ValidationError::Custom {
                        message: "ffprobe not available but required for duration detection"
                            .to_string(),
                    })
                }
            }
            Err(_) => Err(ValidationError::Custom {
                message: "ffprobe not found in PATH".to_string(),
            }),
        }
    }

    pub fn validate_disk_space(
        &self,
        output_path: &Path,
        estimated_size_mb: Option<u64>,
    ) -> Result<(), ValidationError> {
        // This is a simplified implementation
        // In a real application, you would check actual available disk space
        if let Some(_size) = estimated_size_mb {
            // For now, assume we have enough space
            // TODO: Implement actual disk space checking using system APIs
        }

        // Check if parent directory exists
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                return Err(ValidationError::OutputDirectoryNotFound {
                    path: parent.to_string_lossy().to_string(),
                });
            }
        }

        Ok(())
    }

    pub fn validate_all(
        &self,
        input_path: Option<&Path>,
        output_path: Option<&Path>,
        video_codec: &str,
        audio_codec: &str,
        quality: &str,
        container: &str,
    ) -> Result<(), ValidationError> {
        // Validate FFmpeg availability first
        self.validate_ffmpeg_available()?;
        // Make FFprobe validation optional - warn but don't fail
        if let Err(e) = self.validate_ffprobe_available() {
            tracing::warn!("FFprobe validation warning: {}", e);
        }

        // Validate file paths
        self.validate_paths(input_path, output_path)?;

        // Validate conversion settings
        self.validate_conversion_settings(video_codec, audio_codec, quality, container)?;

        // Validate disk space if output path is available
        if let Some(output) = output_path {
            self.validate_disk_space(output, None)?;
        }

        Ok(())
    }
}

impl Default for ValidationService {
    fn default() -> Self {
        Self::new()
    }
}
