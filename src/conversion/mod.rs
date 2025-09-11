use crate::security::SecurityValidator;
use regex::Regex;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ConversionProgress {
    pub percentage: f32,
    pub current_frame: u32,
    pub fps: f32,
    pub speed: f32,
    pub bitrate: String,
    pub time_elapsed: String,
    pub eta: Option<Duration>,
    pub size: String,
    pub total_frames: Option<u32>,
}

impl Default for ConversionProgress {
    fn default() -> Self {
        Self {
            percentage: 0.0,
            current_frame: 0,
            fps: 0.0,
            speed: 0.0,
            bitrate: String::new(),
            time_elapsed: String::new(),
            eta: None,
            size: String::new(),
            total_frames: None,
        }
    }
}

impl PartialEq for ConversionProgress {
    fn eq(&self, other: &Self) -> bool {
        self.current_frame == other.current_frame
            && (self.percentage - other.percentage).abs() < f32::EPSILON
            && (self.fps - other.fps).abs() < f32::EPSILON
            && (self.speed - other.speed).abs() < f32::EPSILON
            && self.bitrate == other.bitrate
            && self.time_elapsed == other.time_elapsed
            && self.eta == other.eta
            && self.size == other.size
            && self.total_frames == other.total_frames
    }
}

#[derive(Debug, Clone)]
pub enum ConversionStatus {
    Starting,
    InProgress(ConversionProgress),
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("FFmpeg not found or not accessible")]
    FFmpegNotFound,
    #[error("Input file validation failed: {message}")]
    InvalidInput { message: String },
    #[error("Security validation failed: {message}")]
    SecurityError { message: String },
    #[error("Process execution failed: {message}")]
    ProcessError { message: String },
    #[error("Conversion was cancelled by user")]
    Cancelled,
    #[error("IO error occurred: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Invalid conversion settings: {message}")]
    InvalidSettings { message: String },
}

impl ConversionError {
    pub fn user_message(&self) -> String {
        match self {
            Self::FFmpegNotFound => {
                "FFmpeg not found. Please install FFmpeg and ensure it's in your PATH.".to_string()
            }
            Self::InvalidInput { message } => format!("Invalid input: {}", message),
            Self::SecurityError { message } => {
                format!("Security validation failed: {}", message)
            }
            Self::ProcessError { message } => format!("Process error: {}", message),
            Self::Cancelled => "Conversion was cancelled by user.".to_string(),
            Self::Io { source } => format!("File operation failed: {}", source),
            Self::InvalidSettings { message } => format!("Invalid settings: {}", message),
        }
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::InvalidInput { .. }
                | Self::InvalidSettings { .. }
                | Self::SecurityError { .. }
                | Self::Cancelled
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionSettings {
    pub mode: ConversionMode,
    pub video_codec: String,
    pub audio_codec: String,
    pub quality: String,
    pub use_hardware_accel: bool,
    pub container: String,
}

impl Default for ConversionSettings {
    fn default() -> Self {
        Self {
            mode: ConversionMode::default(),
            video_codec: "libx264".to_string(),
            audio_codec: "aac".to_string(),
            quality: "23".to_string(),
            use_hardware_accel: true,
            container: "mp4".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum ConversionMode {
    #[default]
    Convert,
    Remux,
}

#[derive(Debug, Clone)]
pub struct ConversionTask {
    pub id: Uuid,
    pub input: String,
    pub output: String,
    pub settings: ConversionSettings,
    duration_seconds: Option<f32>,
    cancel_flag: Arc<Mutex<bool>>,
    security_validator: SecurityValidator,
}

impl ConversionTask {
    pub fn new(input: String, output: String, settings: ConversionSettings) -> Self {
        Self {
            id: Uuid::new_v4(),
            input,
            output,
            settings,
            duration_seconds: None,
            cancel_flag: Arc::new(Mutex::new(false)),
            security_validator: SecurityValidator::new(),
        }
    }

    pub fn new_with_id(
        id: Uuid,
        input: String,
        output: String,
        settings: ConversionSettings,
    ) -> Self {
        Self {
            id,
            input,
            output,
            settings,
            duration_seconds: None,
            cancel_flag: Arc::new(Mutex::new(false)),
            security_validator: SecurityValidator::new(),
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn cancel(&self) {
        if let Ok(mut flag) = self.cancel_flag.lock() {
            *flag = true;
        }
    }

    pub fn is_cancelled(&self) -> bool {
        if let Ok(flag) = self.cancel_flag.lock() {
            *flag
        } else {
            false
        }
    }

    pub fn validate(&self) -> Result<(), ConversionError> {
        if self.input.is_empty() {
            return Err(ConversionError::InvalidInput {
                message: "No input file specified".to_string(),
            });
        }

        if !std::path::Path::new(&self.input).exists() {
            return Err(ConversionError::InvalidInput {
                message: "Input file does not exist".to_string(),
            });
        }

        if self.output.is_empty() {
            return Err(ConversionError::InvalidInput {
                message: "No output file specified".to_string(),
            });
        }

        if let Some(parent) = std::path::Path::new(&self.output).parent() {
            if !parent.exists() {
                return Err(ConversionError::InvalidInput {
                    message: "Output directory does not exist".to_string(),
                });
            }
        }

        // Validate paths with security validator
        self.security_validator
            .validate_path(&self.input)
            .map_err(|e| ConversionError::SecurityError {
                message: e.to_string(),
            })?;

        self.security_validator
            .validate_path(&self.output)
            .map_err(|e| ConversionError::SecurityError {
                message: e.to_string(),
            })?;

        Ok(())
    }

    async fn get_duration(&mut self) -> Result<(), ConversionError> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-show_entries",
                "format=duration",
                "-of",
                "csv=p=0",
                &self.input,
            ])
            .output()
            .await
            .map_err(|_| ConversionError::FFmpegNotFound)?;

        if output.status.success() {
            if let Ok(duration_str) = String::from_utf8(output.stdout) {
                if let Ok(duration) = duration_str.trim().parse::<f32>() {
                    self.duration_seconds = Some(duration);
                }
            }
        }

        Ok(())
    }

    async fn get_frame_count(&mut self) -> Result<Option<u32>, ConversionError> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-select_streams",
                "v:0",
                "-count_frames",
                "-show_entries",
                "stream=nb_frames",
                "-csv=p=0",
                &self.input,
            ])
            .output()
            .await
            .map_err(|_| ConversionError::FFmpegNotFound)?;

        if output.status.success() {
            if let Ok(frame_str) = String::from_utf8(output.stdout) {
                if let Ok(frames) = frame_str.trim().parse::<u32>() {
                    return Ok(Some(frames));
                }
            }
        }

        Ok(None)
    }

    pub async fn execute(
        &mut self,
    ) -> Result<tokio::sync::mpsc::Receiver<ConversionStatus>, ConversionError> {
        self.validate()?;
        let _ = self.get_duration().await;
        let total_frames = self.get_frame_count().await.ok().flatten();

        // Build secure FFmpeg command
        let args = self
            .security_validator
            .build_safe_ffmpeg_command(
                &self.input,
                &self.output,
                &self.settings.video_codec,
                &self.settings.audio_codec,
                &self.settings.quality,
                self.settings.use_hardware_accel,
                self.settings.mode == ConversionMode::Remux,
            )
            .map_err(|e| ConversionError::SecurityError {
                message: e.to_string(),
            })?;

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let duration = self.duration_seconds;
        let cancel_flag = self.cancel_flag.clone();
        let task_id = self.id;

        tokio::spawn(async move {
            std::mem::drop(tx.send(ConversionStatus::Starting));

            // Start FFmpeg process
            let mut child = match Command::new("ffmpeg")
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    let error_msg = if e.kind() == std::io::ErrorKind::NotFound {
                        "FFmpeg not found. Please install FFmpeg and ensure it's in your PATH."
                            .to_string()
                    } else {
                        format!("Failed to start FFmpeg: {}", e)
                    };
                    let _ = tx.send(ConversionStatus::Failed(error_msg)).await;
                    return;
                }
            };

            let child_id = child.id();
            let stdout = child
                .stdout
                .take()
                .expect("Failed to capture stdout from FFmpeg");

            let stderr = child
                .stderr
                .take()
                .expect("Failed to capture stderr from FFmpeg");

            let cancel_flag_clone = cancel_flag.clone();
            let tx_clone = tx.clone();

            // Monitor cancellation in a separate task
            let cancellation_task = tokio::spawn(async move {
                loop {
                    let cancelled = {
                        let guard = cancel_flag_clone.lock().unwrap_or_else(|e| e.into_inner());
                        *guard
                    };

                    if cancelled {
                        // Kill the process
                        if let Some(pid) = child_id {
                            #[cfg(unix)]
                            {
                                let _ = Command::new("kill")
                                    .args(["-TERM", &pid.to_string()])
                                    .output()
                                    .await;
                            }
                            #[cfg(windows)]
                            {
                                let _ = Command::new("taskkill")
                                    .args(["/F", "/PID", &pid.to_string()])
                                    .output()
                                    .await;
                            }
                        }
                        let _ = tx_clone.send(ConversionStatus::Cancelled).await;
                        break;
                    }

                    tokio::time::sleep(Duration::from_millis(
                        crate::constants::CANCELLATION_CHECK_INTERVAL_MS,
                    ))
                    .await;
                }
            });

            // Parse FFmpeg output for progress
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            let progress_parser = ProgressParser::new(duration, total_frames);

            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break; // End of stream
                }

                // Check for cancellation
                let cancelled = {
                    let guard = cancel_flag.lock().unwrap_or_else(|e| e.into_inner());
                    *guard
                };
                if cancelled {
                    break;
                }

                // Parse progress and send update
                if let Some(progress) = progress_parser.parse_line(&line) {
                    let _ = tx.send(ConversionStatus::InProgress(progress));
                }

                line.clear();
            }

            // Cancel the cancellation monitoring task
            cancellation_task.abort();

            // Check if we were cancelled
            let cancelled = {
                let guard = cancel_flag.lock().unwrap_or_else(|e| e.into_inner());
                *guard
            };
            if cancelled {
                return;
            }

            // Wait for the process to complete
            match child.wait().await {
                Ok(status) => {
                    if status.success() {
                        let _ = tx.send(ConversionStatus::Completed).await;
                        tracing::info!("Conversion {} completed successfully", task_id);
                    } else {
                        // Capture stderr for error details
                        let mut stderr_reader = BufReader::new(stderr);
                        let mut error_output = String::new();
                        let mut error_line = String::new();
                        let mut line_count = 0;

                        while line_count < 10 {
                            // Limit error output
                            if let Ok(bytes) = stderr_reader.read_line(&mut error_line).await {
                                if bytes == 0 {
                                    break;
                                }
                                error_output.push_str(&error_line);
                                error_line.clear();
                                line_count += 1;
                            } else {
                                break;
                            }
                        }

                        let error_msg = if error_output.is_empty() {
                            "FFmpeg process failed without specific error message".to_string()
                        } else {
                            format!("FFmpeg error: {}", error_output.trim())
                        };

                        let _ = tx.send(ConversionStatus::Failed(error_msg)).await;
                        tracing::error!("Conversion {} failed: {}", task_id, error_output);
                    }
                }
                Err(e) => {
                    let error_msg = format!("Process execution error: {}", e);
                    let _ = tx.send(ConversionStatus::Failed(error_msg.clone())).await;
                    tracing::error!("Conversion {} process error: {}", task_id, error_msg);
                }
            }
        });

        Ok(rx)
    }
}

struct ProgressParser {
    duration_seconds: Option<f32>,
    total_frames: Option<u32>,
    frame_regex: Regex,
    fps_regex: Regex,
    time_regex: Regex,
    speed_regex: Regex,
    bitrate_regex: Regex,
    size_regex: Regex,
}

impl ProgressParser {
    fn new(duration_seconds: Option<f32>, total_frames: Option<u32>) -> Self {
        Self {
            duration_seconds,
            total_frames,
            frame_regex: Regex::new(r"frame=\s*(\d+)").unwrap(),
            fps_regex: Regex::new(r"fps=\s*([\d.]+)").unwrap(),
            time_regex: Regex::new(r"time=(\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap(),
            speed_regex: Regex::new(r"speed=\s*([\d.]+)x").unwrap(),
            bitrate_regex: Regex::new(r"bitrate=\s*([\d.]+\w+/s)").unwrap(),
            size_regex: Regex::new(r"size=\s*(\d+\w+)").unwrap(),
        }
    }

    fn parse_line(&self, line: &str) -> Option<ConversionProgress> {
        if !line.contains("frame=") {
            return None;
        }

        let mut progress = ConversionProgress {
            total_frames: self.total_frames,
            ..Default::default()
        };

        if let Some(caps) = self.frame_regex.captures(line) {
            progress.current_frame = caps[1].parse().unwrap_or(0);
        }

        if let Some(caps) = self.fps_regex.captures(line) {
            progress.fps = caps[1].parse().unwrap_or(0.0);
        }

        if let Some(caps) = self.speed_regex.captures(line) {
            progress.speed = caps[1].parse().unwrap_or(0.0);
        }

        if let Some(caps) = self.bitrate_regex.captures(line) {
            progress.bitrate = caps[1].to_string();
        }

        if let Some(caps) = self.size_regex.captures(line) {
            progress.size = caps[1].to_string();
        }

        if let Some(caps) = self.time_regex.captures(line) {
            let hours: f32 = caps[1].parse().unwrap_or(0.0);
            let minutes: f32 = caps[2].parse().unwrap_or(0.0);
            let seconds: f32 = caps[3].parse().unwrap_or(0.0);
            let centiseconds: f32 = caps[4].parse().unwrap_or(0.0);

            let total_seconds = hours * 3600.0 + minutes * 60.0 + seconds + centiseconds / 100.0;
            progress.time_elapsed = format!(
                "{:02}:{:02}:{:05.2}",
                hours as u32,
                minutes as u32,
                seconds + centiseconds / 100.0
            );

            // Calculate percentage based on duration or frame count
            if let Some(duration) = self.duration_seconds {
                progress.percentage = ((total_seconds / duration) * 100.0).clamp(0.0, 100.0);

                // Calculate ETA
                if progress.speed > 0.0 && total_seconds > 0.0 {
                    let remaining_seconds = (duration - total_seconds) / progress.speed;
                    if remaining_seconds > 0.0 && remaining_seconds.is_finite() {
                        progress.eta = Some(Duration::from_secs_f32(remaining_seconds));
                    }
                }
            } else if let Some(total_frames) = self.total_frames {
                if total_frames > 0 {
                    progress.percentage = ((progress.current_frame as f32 / total_frames as f32)
                        * 100.0)
                        .clamp(0.0, 100.0);
                }
            }
        }

        Some(progress)
    }
}
