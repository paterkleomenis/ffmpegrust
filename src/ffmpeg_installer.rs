use anyhow::{anyhow, Result};

use std::process::Command;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub enum InstallStatus {
    AlreadyInstalled,
    Installing,
    InstallSuccess,
    InstallFailed(String),
    NotSupported,
}

pub struct FFmpegInstaller;

impl FFmpegInstaller {
    pub fn new() -> Self {
        Self
    }

    /// Check if FFmpeg is already installed and accessible
    pub fn is_ffmpeg_installed() -> bool {
        match Command::new("ffmpeg").arg("-version").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Get FFmpeg version if installed
    pub fn get_ffmpeg_version() -> Option<String> {
        if !Self::is_ffmpeg_installed() {
            return None;
        }

        match Command::new("ffmpeg").arg("-version").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse version from first line: "ffmpeg version 4.4.2-0ubuntu0.22.04.1"
                if let Some(line) = stdout.lines().next() {
                    if let Some(version_part) = line.split_whitespace().nth(2) {
                        return Some(version_part.to_string());
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    /// Install FFmpeg on the current platform
    pub async fn install_ffmpeg() -> Result<InstallStatus> {
        if Self::is_ffmpeg_installed() {
            info!("FFmpeg is already installed");
            return Ok(InstallStatus::AlreadyInstalled);
        }

        info!("Starting FFmpeg installation...");

        #[cfg(target_os = "windows")]
        return Self::install_windows().await;

        #[cfg(target_os = "macos")]
        return Self::install_macos().await;

        #[cfg(target_os = "linux")]
        return Self::install_linux().await;

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            warn!("FFmpeg auto-installation not supported on this platform");
            Ok(InstallStatus::NotSupported)
        }
    }

    #[cfg(target_os = "windows")]
    async fn install_windows() -> Result<InstallStatus> {
        info!("Installing FFmpeg on Windows using winget...");

        // First try winget
        match Command::new("winget")
            .args(&[
                "install",
                "--id=Gyan.FFmpeg",
                "-e",
                "--accept-source-agreements",
                "--accept-package-agreements",
            ])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("FFmpeg installed successfully via winget");
                    return Ok(InstallStatus::InstallSuccess);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Winget installation failed: {}", stderr);
                }
            }
            Err(e) => {
                warn!("Winget not available: {}", e);
            }
        }

        // Fallback: Try chocolatey
        match Command::new("choco")
            .args(&["install", "ffmpeg", "-y"])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("FFmpeg installed successfully via chocolatey");
                    return Ok(InstallStatus::InstallSuccess);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Chocolatey installation failed: {}", stderr);
                }
            }
            Err(e) => {
                warn!("Chocolatey not available: {}", e);
            }
        }

        // If both fail, provide manual installation instructions
        let error_msg = "Automatic installation failed. Please install FFmpeg manually:\n\
                        1. Download from: https://www.gyan.dev/ffmpeg/builds/\n\
                        2. Extract to a folder (e.g., C:\\ffmpeg)\n\
                        3. Add C:\\ffmpeg\\bin to your PATH environment variable\n\
                        4. Restart this application";

        error!("{}", error_msg);
        Ok(InstallStatus::InstallFailed(error_msg.to_string()))
    }

    #[cfg(target_os = "macos")]
    async fn install_macos() -> Result<InstallStatus> {
        info!("Installing FFmpeg on macOS using Homebrew...");

        // Check if Homebrew is installed
        if !Self::is_homebrew_installed() {
            info!("Homebrew not found, installing Homebrew first...");
            if let Err(e) = Self::install_homebrew().await {
                let error_msg = format!(
                    "Failed to install Homebrew: {}. Please install manually from https://brew.sh",
                    e
                );
                error!("{}", error_msg);
                return Ok(InstallStatus::InstallFailed(error_msg));
            }
        }

        // Install FFmpeg via Homebrew
        match Command::new("brew").args(&["install", "ffmpeg"]).output() {
            Ok(output) => {
                if output.status.success() {
                    info!("FFmpeg installed successfully via Homebrew");
                    Ok(InstallStatus::InstallSuccess)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let error_msg = format!(
                        "Homebrew installation failed: {}. Please try: brew install ffmpeg",
                        stderr
                    );
                    error!("{}", error_msg);
                    Ok(InstallStatus::InstallFailed(error_msg))
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to run brew command: {}. Please install FFmpeg manually: brew install ffmpeg", e);
                error!("{}", error_msg);
                Ok(InstallStatus::InstallFailed(error_msg))
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn is_homebrew_installed() -> bool {
        Command::new("brew").arg("--version").output().is_ok()
    }

    #[cfg(target_os = "macos")]
    async fn install_homebrew() -> Result<()> {
        let install_script = r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#;

        let output = Command::new("bash")
            .args(&["-c", install_script])
            .output()
            .map_err(|e| anyhow!("Failed to execute Homebrew installation: {}", e))?;

        if output.status.success() {
            info!("Homebrew installed successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Homebrew installation failed: {}", stderr))
        }
    }

    #[cfg(target_os = "linux")]
    async fn install_linux() -> Result<InstallStatus> {
        info!("Installing FFmpeg on Linux...");

        // Try different package managers in order of preference
        let package_managers = [
            (
                "apt",
                vec!["update", "&&", "apt", "install", "-y", "ffmpeg"],
            ),
            ("dnf", vec!["install", "-y", "ffmpeg"]),
            ("yum", vec!["install", "-y", "ffmpeg"]),
            ("pacman", vec!["-S", "--noconfirm", "ffmpeg"]),
            ("zypper", vec!["install", "-y", "ffmpeg"]),
        ];

        for (pm, args) in &package_managers {
            if Self::is_command_available(pm) {
                info!("Trying to install FFmpeg using {}...", pm);

                let command = if *pm == "apt" {
                    // Special handling for apt update && apt install
                    Command::new("sh")
                        .args(&["-c", "apt update && apt install -y ffmpeg"])
                        .output()
                } else {
                    Command::new(*pm).args(args).output()
                };

                match command {
                    Ok(output) => {
                        if output.status.success() {
                            info!("FFmpeg installed successfully using {}", pm);
                            return Ok(InstallStatus::InstallSuccess);
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            warn!("Installation failed with {}: {}", pm, stderr);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to run {} command: {}", pm, e);
                    }
                }
            }
        }

        // If all package managers fail, provide manual instructions
        let error_msg = "Automatic installation failed. Please install FFmpeg manually using your distribution's package manager:\n\
                        • Ubuntu/Debian: sudo apt update && sudo apt install ffmpeg\n\
                        • Fedora/RHEL: sudo dnf install ffmpeg\n\
                        • Arch Linux: sudo pacman -S ffmpeg\n\
                        • openSUSE: sudo zypper install ffmpeg";

        error!("{}", error_msg);
        Ok(InstallStatus::InstallFailed(error_msg.to_string()))
    }

    fn is_command_available(command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Show installation instructions for manual installation
    pub fn get_manual_installation_instructions() -> String {
        #[cfg(target_os = "windows")]
        return "Windows Installation:\n\
                1. Download FFmpeg from: https://www.gyan.dev/ffmpeg/builds/\n\
                2. Extract to C:\\ffmpeg\n\
                3. Add C:\\ffmpeg\\bin to your PATH\n\
                4. Restart this application\n\n\
                Alternative: Install via winget or chocolatey:\n\
                • winget install Gyan.FFmpeg\n\
                • choco install ffmpeg"
            .to_string();

        #[cfg(target_os = "macos")]
        return "macOS Installation:\n\
                1. Install Homebrew: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\n\
                2. Install FFmpeg: brew install ffmpeg\n\n\
                Alternative: Download from https://evermeet.cx/ffmpeg/".to_string();

        #[cfg(target_os = "linux")]
        return "Linux Installation:\n\
                Choose the command for your distribution:\n\
                • Ubuntu/Debian: sudo apt update && sudo apt install ffmpeg\n\
                • Fedora/RHEL: sudo dnf install ffmpeg\n\
                • Arch Linux: sudo pacman -S ffmpeg\n\
                • openSUSE: sudo zypper install ffmpeg"
            .to_string();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return "Please install FFmpeg manually for your operating system.\n\
                Visit https://ffmpeg.org/download.html for download links."
            .to_string();
    }

    /// Check FFmpeg capabilities (codecs, formats, etc.)
    pub fn check_ffmpeg_capabilities() -> Result<FFmpegCapabilities> {
        if !Self::is_ffmpeg_installed() {
            return Err(anyhow!("FFmpeg is not installed"));
        }

        let mut capabilities = FFmpegCapabilities::default();

        // Check for hardware acceleration support
        if let Ok(output) = Command::new("ffmpeg").args(&["-encoders"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            capabilities.has_nvenc = stdout.contains("h264_nvenc") || stdout.contains("hevc_nvenc");
            capabilities.has_qsv = stdout.contains("h264_qsv") || stdout.contains("hevc_qsv");
            capabilities.has_vaapi = stdout.contains("h264_vaapi") || stdout.contains("hevc_vaapi");
            capabilities.has_videotoolbox =
                stdout.contains("h264_videotoolbox") || stdout.contains("hevc_videotoolbox");
        }

        // Check for common formats
        if let Ok(output) = Command::new("ffmpeg").args(&["-formats"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            capabilities.supports_mp4 = stdout.contains("mp4");
            capabilities.supports_mkv = stdout.contains("matroska");
            capabilities.supports_webm = stdout.contains("webm");
        }

        Ok(capabilities)
    }
}

#[derive(Debug, Default)]
pub struct FFmpegCapabilities {
    pub has_nvenc: bool,
    pub has_qsv: bool,
    pub has_vaapi: bool,
    pub has_videotoolbox: bool,
    pub supports_mp4: bool,
    pub supports_mkv: bool,
    pub supports_webm: bool,
}

impl FFmpegCapabilities {
    pub fn get_recommended_encoder(&self) -> &'static str {
        #[cfg(target_os = "windows")]
        {
            if self.has_nvenc {
                return "h264_nvenc";
            }
            if self.has_qsv {
                return "h264_qsv";
            }
        }

        #[cfg(target_os = "macos")]
        {
            if self.has_videotoolbox {
                return "h264_videotoolbox";
            }
        }

        #[cfg(target_os = "linux")]
        {
            if self.has_nvenc {
                return "h264_nvenc";
            }
            if self.has_vaapi {
                return "h264_vaapi";
            }
            if self.has_qsv {
                return "h264_qsv";
            }
        }

        "libx264" // fallback to software encoding
    }

    pub fn hardware_acceleration_available(&self) -> bool {
        self.has_nvenc || self.has_qsv || self.has_vaapi || self.has_videotoolbox
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_detection() {
        // This test will pass if FFmpeg is installed on the test system
        let installed = FFmpegInstaller::is_ffmpeg_installed();
        println!("FFmpeg installed: {}", installed);
    }

    #[test]
    fn test_version_detection() {
        if FFmpegInstaller::is_ffmpeg_installed() {
            let version = FFmpegInstaller::get_ffmpeg_version();
            println!("FFmpeg version: {:?}", version);
        }
    }

    #[test]
    fn test_manual_instructions() {
        let instructions = FFmpegInstaller::get_manual_installation_instructions();
        assert!(!instructions.is_empty());
        println!("Manual installation instructions:\n{}", instructions);
    }
}
