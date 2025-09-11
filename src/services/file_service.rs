use crate::constants::VIDEO_EXTENSIONS;
use crate::events::EventSender;
use crate::services::Service;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("File not found: {path}")]
    NotFound { path: String },
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    #[error("Invalid file format: {path}")]
    InvalidFormat { path: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct FileService {
    event_sender: EventSender,
}

impl FileService {
    pub fn new(event_sender: EventSender) -> Self {
        Self { event_sender }
    }

    pub async fn validate_input_file(&self, path: &Path) -> Result<(), FileError> {
        if !path.exists() {
            return Err(FileError::NotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        if !path.is_file() {
            return Err(FileError::InvalidFormat {
                path: format!("{} is not a file", path.display()),
            });
        }

        // Check if file is readable
        let metadata = tokio::fs::metadata(path).await?;
        if metadata.len() == 0 {
            return Err(FileError::InvalidFormat {
                path: format!("{} is empty", path.display()),
            });
        }

        // Validate file format
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if !VIDEO_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()) {
                    return Err(FileError::InvalidFormat {
                        path: format!("Unsupported file format: {}", ext_str),
                    });
                }
            }
        }

        // Use infer crate to detect actual file type
        let buffer = tokio::fs::read(path).await?;
        let first_bytes = &buffer[..std::cmp::min(buffer.len(), 8192)];

        if let Some(kind) = infer::get(first_bytes) {
            match kind.mime_type() {
                mime if mime.starts_with("video/") => Ok(()),
                _ => Err(FileError::InvalidFormat {
                    path: format!("File is not a video: detected {}", kind.mime_type()),
                }),
            }
        } else {
            // If infer can't detect the type, trust the extension for now
            Ok(())
        }
    }

    pub async fn validate_output_file(&self, path: &Path) -> Result<(), FileError> {
        // Check if parent directory exists and is writable
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(FileError::NotFound {
                    path: format!("Output directory does not exist: {}", parent.display()),
                });
            }

            // Test write permissions by creating a temporary file
            let test_file = parent.join(".write_test");
            match tokio::fs::write(&test_file, b"test").await {
                Ok(_) => {
                    let _ = tokio::fs::remove_file(&test_file).await;
                }
                Err(_) => {
                    return Err(FileError::PermissionDenied {
                        path: parent.to_string_lossy().to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn auto_generate_output_path(
        &self,
        input_path: &Path,
        suffix: &str,
        extension: &str,
    ) -> Option<PathBuf> {
        if let Some(parent) = input_path.parent() {
            if let Some(stem) = input_path.file_stem() {
                let output_name = format!("{}{}.{}", stem.to_string_lossy(), suffix, extension);
                return Some(parent.join(output_name));
            }
        }
        None
    }

    pub fn get_file_info(&self, path: &Path) -> Result<FileInfo, FileError> {
        if !path.exists() {
            return Err(FileError::NotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        let modified = metadata.modified().ok();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(FileInfo {
            path: path.to_path_buf(),
            size,
            extension,
            modified,
        })
    }

    pub async fn ensure_unique_output_path(&self, path: PathBuf) -> PathBuf {
        if !path.exists() {
            return path;
        }

        let parent = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let mut counter = 1;
        loop {
            let new_name = if extension.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, extension)
            };

            let new_path = parent.join(new_name);

            if !new_path.exists() {
                return new_path;
            }

            counter += 1;

            // Prevent infinite loop
            if counter > 1000 {
                return parent.join(format!("{}_{}", stem, counter));
            }
        }
    }
}

#[async_trait::async_trait]
impl Service for FileService {
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize any file service resources
        tracing::info!("File service initialized");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("File service shutdown");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub extension: String,
    pub modified: Option<std::time::SystemTime>,
}

impl FileInfo {
    pub fn size_human_readable(&self) -> String {
        let size = self.size as f64;
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

        let mut size = size;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    pub fn modified_human_readable(&self) -> String {
        if let Some(modified) = self.modified {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                let timestamp = duration.as_secs();
                // This is a simplified timestamp formatting
                // In a real application, you'd use a proper datetime library
                format!("Modified: {}", timestamp)
            } else {
                "Unknown modification time".to_string()
            }
        } else {
            "Unknown modification time".to_string()
        }
    }
}
