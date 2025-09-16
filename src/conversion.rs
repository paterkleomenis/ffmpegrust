use crate::presets::{AudioCodec, ConversionMode, ConversionPreset, VideoCodec, VideoFormat};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone)]
pub struct ConversionProgress {
    pub percentage: f32,
    pub time_remaining: Option<Duration>,
    pub current_time: String,
    pub total_time: String,
}

#[derive(Debug, Clone)]
pub enum ConversionMessage {
    Progress(ConversionProgress),
    Completed(PathBuf),
    Error(String),
}

pub struct ConversionTask {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub preset: ConversionPreset,
    pub sender: Sender<ConversionMessage>,
}

impl ConversionTask {
    pub fn new(
        input_file: PathBuf,
        output_file: PathBuf,
        preset: ConversionPreset,
        sender: Sender<ConversionMessage>,
    ) -> Self {
        Self {
            input_file,
            output_file,
            preset,
            sender,
        }
    }

    pub async fn execute(self) {
        let result = self.run_conversion().await;

        match result {
            Ok(output_path) => {
                let _ = self.sender.send(ConversionMessage::Completed(output_path));
            }
            Err(error) => {
                let _ = self.sender.send(ConversionMessage::Error(error));
            }
        }
    }

    async fn run_conversion(&self) -> Result<PathBuf, String> {
        // Build FFmpeg command
        let mut cmd = AsyncCommand::new("ffmpeg");
        cmd.arg("-i")
            .arg(&self.input_file)
            .arg("-y") // Overwrite output file
            .arg("-progress")
            .arg("pipe:2") // Send progress to stderr
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        // Add codec arguments based on preset
        match self.preset.mode {
            ConversionMode::Convert => {
                // Video codec
                if self.preset.video_codec != VideoCodec::Copy {
                    cmd.arg("-c:v").arg(self.preset.video_codec.ffmpeg_name());

                    // Video bitrate
                    if let Some(ref bitrate) = self.preset.video_bitrate {
                        if !bitrate.is_empty() {
                            cmd.arg("-b:v").arg(bitrate);
                        }
                    }

                    // Resolution
                    if let Some(ref resolution) = self.preset.resolution {
                        if !resolution.is_empty() {
                            cmd.arg("-s").arg(resolution);
                        }
                    }

                    // Frame rate
                    if let Some(ref frame_rate) = self.preset.frame_rate {
                        if !frame_rate.is_empty() {
                            cmd.arg("-r").arg(frame_rate);
                        }
                    }
                } else {
                    cmd.arg("-c:v").arg("copy");
                }

                // Audio codec
                if self.preset.audio_codec != AudioCodec::Copy {
                    cmd.arg("-c:a").arg(self.preset.audio_codec.ffmpeg_name());

                    // Audio bitrate
                    if let Some(ref bitrate) = self.preset.audio_bitrate {
                        if !bitrate.is_empty() {
                            cmd.arg("-b:a").arg(bitrate);
                        }
                    }
                } else {
                    cmd.arg("-c:a").arg("copy");
                }
            }
            ConversionMode::Remux => {
                // Just copy streams for remuxing
                cmd.arg("-c").arg("copy");

                // Handle metadata options
                self.apply_metadata_options(&mut cmd);
            }
        }

        cmd.arg(&self.output_file);

        // Get total duration first
        let total_duration = self.get_video_duration().await?;

        // Start the conversion process
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start FFmpeg: {}", e))?;

        let stderr = child
            .stderr
            .take()
            .ok_or("Failed to capture FFmpeg stderr")?;

        let mut reader = BufReader::new(stderr).lines();
        let start_time = Instant::now();

        // Parse progress output
        while let Ok(Some(line)) = reader.next_line().await {
            if line.starts_with("out_time_ms=") {
                if let Some(time_str) = line.strip_prefix("out_time_ms=") {
                    if let Ok(time_microseconds) = time_str.parse::<u64>() {
                        let current_time_seconds = time_microseconds as f64 / 1_000_000.0;
                        let percentage = if total_duration > 0.0 {
                            (current_time_seconds / total_duration * 100.0) as f32
                        } else {
                            0.0
                        };

                        let elapsed = start_time.elapsed();
                        let time_remaining = if percentage > 0.0 {
                            let estimated_total = elapsed.as_secs_f64() * 100.0 / percentage as f64;
                            let remaining = estimated_total - elapsed.as_secs_f64();
                            if remaining > 0.0 {
                                Some(Duration::from_secs_f64(remaining))
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        let progress = ConversionProgress {
                            percentage: percentage.min(100.0),
                            time_remaining,
                            current_time: format_duration(current_time_seconds),
                            total_time: format_duration(total_duration),
                        };

                        let _ = self.sender.send(ConversionMessage::Progress(progress));
                    }
                }
            }
        }

        // Wait for the process to complete
        let status = child
            .wait()
            .await
            .map_err(|e| format!("Failed to wait for FFmpeg process: {}", e))?;

        if status.success() {
            Ok(self.output_file.clone())
        } else {
            Err("FFmpeg conversion failed".to_string())
        }
    }

    async fn get_video_duration(&self) -> Result<f64, String> {
        let output = AsyncCommand::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-show_entries")
            .arg("format=duration")
            .arg("-of")
            .arg("csv=p=0")
            .arg(&self.input_file)
            .output()
            .await
            .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

        let duration_str = String::from_utf8_lossy(&output.stdout);
        duration_str
            .trim()
            .parse::<f64>()
            .map_err(|_| "Failed to parse video duration".to_string())
    }
}

pub fn check_ffmpeg_installation() -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| "FFmpeg not found in PATH".to_string())?;

    if output.status.success() {
        let version_info = String::from_utf8_lossy(&output.stdout);
        if let Some(first_line) = version_info.lines().next() {
            Ok(first_line.to_string())
        } else {
            Err("Unable to parse FFmpeg version".to_string())
        }
    } else {
        Err("FFmpeg command failed".to_string())
    }
}

pub fn generate_output_filename(input_file: &PathBuf, format: &VideoFormat) -> PathBuf {
    let mut output = input_file.clone();
    output.set_extension(format.extension());

    // If the extension is the same, add "_converted" to avoid overwriting
    if output == *input_file {
        let stem = input_file.file_stem().unwrap_or_default();
        let new_name = format!(
            "{}_converted.{}",
            stem.to_string_lossy(),
            format.extension()
        );
        output.set_file_name(new_name);
    }

    output
}

impl ConversionTask {
    fn apply_metadata_options(&self, cmd: &mut AsyncCommand) {
        let metadata = &self.preset.metadata_options;

        if !metadata.copy_file_metadata {
            // Clear file-level metadata
            cmd.arg("-map_metadata").arg("-1");
        }

        if !metadata.copy_chapters {
            // Remove chapters
            cmd.arg("-map_chapters").arg("-1");
        }

        if !metadata.copy_attachments {
            // Exclude attachments (fonts, cover art, etc.)
            cmd.arg("-map").arg("-0:t");
        }

        // Set stream languages if specified
        if !metadata.video_language.is_empty() && metadata.video_language != "und" {
            cmd.arg("-metadata:s:v:0")
                .arg(format!("language={}", metadata.video_language));
        }

        if !metadata.audio_language.is_empty() && metadata.audio_language != "und" {
            cmd.arg("-metadata:s:a:0")
                .arg(format!("language={}", metadata.audio_language));
        }

        if !metadata.subtitle_language.is_empty() && metadata.subtitle_language != "und" {
            cmd.arg("-metadata:s:s:0")
                .arg(format!("language={}", metadata.subtitle_language));
        }

        // Set stream titles if specified
        if !metadata.video_title.is_empty() {
            cmd.arg("-metadata:s:v:0")
                .arg(format!("title={}", metadata.video_title));
        }

        if !metadata.audio_title.is_empty() {
            cmd.arg("-metadata:s:a:0")
                .arg(format!("title={}", metadata.audio_title));
        }

        if !metadata.subtitle_title.is_empty() {
            cmd.arg("-metadata:s:s:0")
                .arg(format!("title={}", metadata.subtitle_title));
        }
    }
}

fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}
