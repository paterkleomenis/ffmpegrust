use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use regex::Regex;

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
        }
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

#[derive(Debug)]
pub enum ConversionError {
    FFmpegNotFound,
    InvalidInput(String),
    PermissionDenied,
    DiskSpace,
    Cancelled,
    ProcessError(String),
}

impl ConversionError {
    pub fn user_message(&self) -> String {
        match self {
            Self::FFmpegNotFound => "FFmpeg not found. Please install FFmpeg and ensure it's in your PATH.".to_string(),
            Self::InvalidInput(msg) => format!("Invalid input file: {}", msg),
            Self::PermissionDenied => "Permission denied. Check write permissions for the output directory.".to_string(),
            Self::DiskSpace => "Insufficient disk space for the conversion.".to_string(),
            Self::Cancelled => "Conversion was cancelled by user.".to_string(),
            Self::ProcessError(msg) => format!("Conversion failed: {}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionSettings {
    pub mode: ConversionMode,
    pub video_codec: String,
    pub audio_codec: String,
    pub quality: String,
    pub use_hardware_accel: bool,
    pub smart_copy: bool,
    pub container: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionMode {
    Convert,
    Remux,
}

impl Default for ConversionMode {
    fn default() -> Self {
        ConversionMode::Convert
    }
}

pub struct ConversionTask {
    pub input: String,
    pub output: String,
    pub settings: ConversionSettings,
    duration_seconds: Option<f32>,
    cancel_flag: Arc<Mutex<bool>>,
}

impl ConversionTask {
    pub fn new(input: String, output: String, settings: ConversionSettings) -> Self {
        Self {
            input,
            output,
            settings,
            duration_seconds: None,
            cancel_flag: Arc::new(Mutex::new(false)),
        }
    }

    pub fn get_cancel_handle(&self) -> Arc<Mutex<bool>> {
        self.cancel_flag.clone()
    }

    pub fn cancel(&self) {
        if let Ok(mut flag) = self.cancel_flag.lock() {
            *flag = true;
        }
    }

    pub fn validate(&self) -> Result<(), ConversionError> {
        if self.input.is_empty() {
            return Err(ConversionError::InvalidInput("No input file selected".to_string()));
        }

        if !std::path::Path::new(&self.input).exists() {
            return Err(ConversionError::InvalidInput("Input file does not exist".to_string()));
        }

        if self.output.is_empty() {
            return Err(ConversionError::InvalidInput("No output file selected".to_string()));
        }

        if let Some(parent) = std::path::Path::new(&self.output).parent() {
            if !parent.exists() {
                return Err(ConversionError::InvalidInput("Output directory does not exist".to_string()));
            }
        }

        Ok(())
    }

    fn get_duration(&mut self) -> Result<(), ConversionError> {
        let output = Command::new("ffprobe")
            .args([
                "-v", "quiet",
                "-show_entries", "format=duration",
                "-of", "csv=p=0",
                &self.input
            ])
            .output()
            .map_err(|_| ConversionError::FFmpegNotFound)?;

        if let Ok(duration_str) = String::from_utf8(output.stdout) {
            if let Ok(duration) = duration_str.trim().parse::<f32>() {
                self.duration_seconds = Some(duration);
            }
        }

        Ok(())
    }

    pub fn execute(&mut self) -> Result<mpsc::Receiver<ConversionStatus>, ConversionError> {
        self.validate()?;
        let _ = self.get_duration();

        let (tx, rx) = mpsc::channel();

        let input = self.input.clone();
        let output = self.output.clone();
        let settings = self.settings.clone();
        let duration = self.duration_seconds;
        let cancel_flag = self.cancel_flag.clone();

        thread::spawn(move || {
            let _ = tx.send(ConversionStatus::Starting);

            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-nostdin")
               .arg("-y")
               .arg("-hide_banner")
               .arg("-loglevel").arg("info")
               .arg("-progress").arg("pipe:1");

            if settings.use_hardware_accel {
                cmd.arg("-hwaccel").arg("auto");
            }

            cmd.arg("-i").arg(&input);

            match settings.mode {
                ConversionMode::Remux => {
                    cmd.arg("-c").arg("copy");
                }
                ConversionMode::Convert => {
                    if settings.smart_copy {
                        if input.ends_with(".webm") {
                            cmd.arg("-c:v").arg("prores_ks")
                               .arg("-profile:v").arg("3")
                               .arg("-c:a").arg("pcm_s16le");
                        } else {
                            cmd.arg("-c:v").arg("copy")
                               .arg("-c:a").arg("pcm_s16le");
                        }
                    } else {
                        cmd.arg("-c:v").arg(&settings.video_codec)
                           .arg("-c:a").arg(&settings.audio_codec);

                        if settings.video_codec.contains("264") || settings.video_codec.contains("265") {
                            cmd.args(["-crf", &settings.quality]);
                        }
                    }
                }
            }

            cmd.arg(&output)
               .stdout(Stdio::piped())
               .stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    let child_id = child.id();
                    
                    // Monitor for cancellation in a separate thread
                    let cancel_monitor = cancel_flag.clone();
                    let tx_cancel = tx.clone();
                    thread::spawn(move || {
                        loop {
                            if let Ok(cancelled) = cancel_monitor.lock() {
                                if *cancelled {
                                    // Kill the process
                                    #[cfg(unix)]
                                    {
                                        let _ = std::process::Command::new("kill")
                                            .arg("-TERM")
                                            .arg(child_id.to_string())
                                            .output();
                                    }
                                    #[cfg(windows)]
                                    {
                                        let _ = std::process::Command::new("taskkill")
                                            .args(["/F", "/PID", &child_id.to_string()])
                                            .output();
                                    }
                                    let _ = tx_cancel.send(ConversionStatus::Cancelled);
                                    break;
                                }
                            }
                            thread::sleep(Duration::from_millis(100));
                        }
                    });

                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        let progress_parser = ProgressParser::new(duration);

                        for line in reader.lines() {
                            // Check for cancellation
                            if let Ok(cancelled) = cancel_flag.lock() {
                                if *cancelled {
                                    break;
                                }
                            }
                            
                            if let Ok(line) = line {
                                if let Some(progress) = progress_parser.parse_line(&line) {
                                    let _ = tx.send(ConversionStatus::InProgress(progress));
                                }
                            }
                        }
                    }

                    // Check if we were cancelled before checking exit status
                    if let Ok(cancelled) = cancel_flag.lock() {
                        if *cancelled {
                            return;
                        }
                    }

                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                let _ = tx.send(ConversionStatus::Completed);
                            } else {
                                // Try to get stderr for more details
                                if let Some(stderr) = child.stderr.take() {
                                    let stderr_reader = BufReader::new(stderr);
                                    let mut error_msg = String::new();
                                    for line in stderr_reader.lines().take(5) {
                                        if let Ok(line) = line {
                                            error_msg.push_str(&line);
                                            error_msg.push('\n');
                                        }
                                    }
                                    let _ = tx.send(ConversionStatus::Failed(format!("FFmpeg error: {}", error_msg)));
                                } else {
                                    let _ = tx.send(ConversionStatus::Failed("Conversion failed".to_string()));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(ConversionStatus::Failed(format!("Process error: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        let _ = tx.send(ConversionStatus::Failed("FFmpeg not found. Please install FFmpeg and ensure it's in your PATH.".to_string()));
                    } else {
                        let _ = tx.send(ConversionStatus::Failed(format!("Failed to start FFmpeg: {}", e)));
                    }
                }
            }
        });

        Ok(rx)
    }
}

struct ProgressParser {
    duration_seconds: Option<f32>,
    frame_regex: Regex,
    fps_regex: Regex,
    time_regex: Regex,
    speed_regex: Regex,
    bitrate_regex: Regex,
    size_regex: Regex,
}

impl ProgressParser {
    fn new(duration_seconds: Option<f32>) -> Self {
        Self {
            duration_seconds,
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

        let mut progress = ConversionProgress::default();

        if let Some(caps) = self.frame_regex.captures(line) {
            progress.current_frame = caps[1].parse().unwrap_or(0);
        }

        if let Some(caps) = self.fps_regex.captures(line) {
            progress.fps = caps[1].parse().unwrap_or(0.0);
        }

        if let Some(caps) = self.time_regex.captures(line) {
            let hours: f32 = caps[1].parse().unwrap_or(0.0);
            let minutes: f32 = caps[2].parse().unwrap_or(0.0);
            let seconds: f32 = caps[3].parse().unwrap_or(0.0);
            let centiseconds: f32 = caps[4].parse().unwrap_or(0.0);

            let total_seconds = hours * 3600.0 + minutes * 60.0 + seconds + centiseconds / 100.0;
            progress.time_elapsed = format!("{}:{:02}:{:05.2}", 
                hours as u32, minutes as u32, seconds + centiseconds / 100.0);

            if let Some(duration) = self.duration_seconds {
                progress.percentage = (total_seconds / duration * 100.0).min(100.0);

                if progress.speed > 0.0 {
                    let remaining_seconds = (duration - total_seconds) / progress.speed;
                    progress.eta = Some(Duration::from_secs_f32(remaining_seconds));
                }
            }
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

        Some(progress)
    }
}