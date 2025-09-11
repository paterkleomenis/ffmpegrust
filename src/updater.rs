use anyhow::{anyhow, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::info;

#[allow(dead_code)]
const GITHUB_API_BASE: &str = "https://api.github.com/repos";
#[allow(dead_code)]
const REPO_OWNER: &str = "paterkleomenis";
#[allow(dead_code)]
const REPO_NAME: &str = "ffmpegrust";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: String,
    pub is_update_available: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    assets: Vec<GitHubAsset>,
    prerelease: bool,
    draft: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    CheckingForUpdates,
    UpdateAvailable(UpdateInfo),
    NoUpdateAvailable,
    DownloadingUpdate(f32), // progress percentage
    InstallingUpdate,
    UpdateCompleted,
    Error(String),
}

pub struct AutoUpdater {
    current_version: Version,
    client: reqwest::Client,
}

impl AutoUpdater {
    pub fn new(current_version: &str) -> Result<Self> {
        let version = Version::parse(current_version)
            .map_err(|e| anyhow!("Invalid version format: {}", e))?;

        let client = reqwest::Client::builder()
            .user_agent(format!("ffmpegrust/{}", current_version))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            current_version: version,
            client,
        })
    }

    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        info!("Checking for updates...");

        let url = format!(
            "{}/{}/{}/releases/latest",
            GITHUB_API_BASE, REPO_OWNER, REPO_NAME
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch release info: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("GitHub API request failed: {}", response.status()));
        }

        let release: GitHubRelease = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse release info: {}", e))?;

        // Skip draft or prerelease versions
        if release.draft || release.prerelease {
            return Ok(UpdateInfo {
                current_version: self.current_version.to_string(),
                latest_version: self.current_version.to_string(),
                download_url: String::new(),
                release_notes: String::new(),
                is_update_available: false,
            });
        }

        // Parse the latest version (remove 'v' prefix if present)
        let latest_version_str = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name);
        let latest_version = Version::parse(latest_version_str)
            .map_err(|e| anyhow!("Invalid latest version format: {}", e))?;

        let is_update_available = latest_version > self.current_version;

        // Find the appropriate asset for the current platform
        let download_url = if is_update_available {
            self.find_platform_asset(&release.assets)?
        } else {
            String::new()
        };

        Ok(UpdateInfo {
            current_version: self.current_version.to_string(),
            latest_version: latest_version.to_string(),
            download_url,
            release_notes: release.body,
            is_update_available,
        })
    }

    fn find_platform_asset(&self, assets: &[GitHubAsset]) -> Result<String> {
        let platform_suffix = self.get_platform_suffix();

        for asset in assets {
            if asset.name.contains(&platform_suffix) {
                return Ok(asset.browser_download_url.clone());
            }
        }

        Err(anyhow!(
            "No asset found for current platform: {}",
            platform_suffix
        ))
    }

    fn get_platform_suffix(&self) -> String {
        #[cfg(target_os = "windows")]
        return "windows-x86_64.zip".to_string();

        #[cfg(target_os = "macos")]
        return "macos-aarch64.tar.gz".to_string();

        #[cfg(target_os = "linux")]
        return "linux-x86_64.tar.gz".to_string();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return "unknown".to_string();
    }

    pub async fn download_and_install_update(
        &self,
        download_url: &str,
        progress_callback: impl Fn(f32) + Send + 'static,
    ) -> Result<()> {
        info!("Starting update download from: {}", download_url);

        // Download the update
        let response = self
            .client
            .get(download_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to start download: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Download failed: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;

        // Create temporary file
        let temp_file =
            NamedTempFile::new().map_err(|e| anyhow!("Failed to create temp file: {}", e))?;
        let temp_path = temp_file.path().to_path_buf();

        // Download with progress tracking
        let mut file = fs::File::create(&temp_path)
            .await
            .map_err(|e| anyhow!("Failed to create download file: {}", e))?;

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow!("Download chunk error: {}", e))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| anyhow!("Failed to write chunk: {}", e))?;

            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let progress = (downloaded as f32 / total_size as f32) * 100.0;
                progress_callback(progress);
            }
        }

        file.flush()
            .await
            .map_err(|e| anyhow!("Failed to flush file: {}", e))?;
        drop(file); // Close the file

        info!("Download completed, starting installation...");

        // Install the update
        self.install_update(&temp_path).await?;

        info!("Update installation completed");
        Ok(())
    }

    async fn install_update(&self, archive_path: &PathBuf) -> Result<()> {
        let current_exe = std::env::current_exe()
            .map_err(|e| anyhow!("Failed to get current executable path: {}", e))?;

        let current_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow!("Failed to get current directory"))?;

        // Create backup of current executable
        let backup_path = current_dir.join("ffmpegrust.backup");
        if backup_path.exists() {
            fs::remove_file(&backup_path).await.ok();
        }
        fs::copy(&current_exe, &backup_path)
            .await
            .map_err(|e| anyhow!("Failed to create backup: {}", e))?;

        // Extract and install based on platform
        #[cfg(target_os = "windows")]
        self.install_windows_update(archive_path, current_dir)
            .await?;

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        self.install_unix_update(archive_path, current_dir).await?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn install_windows_update(
        &self,
        archive_path: &PathBuf,
        install_dir: &std::path::Path,
    ) -> Result<()> {
        // For Windows, we need to use a separate process to replace the executable
        // because we can't replace a running executable on Windows

        let updater_script = install_dir.join("updater.bat");
        let script_content = format!(
            r#"@echo off
timeout /t 2 /nobreak >nul
powershell -Command "Expand-Archive -Path '{}' -DestinationPath '{}' -Force"
copy /Y "{}\\ffmpegrust-windows-x86_64\\ffmpegrust.exe" "{}"
rmdir /S /Q "{}\\ffmpegrust-windows-x86_64"
del "{}"
start "" "{}"
del "%~f0"
"#,
            archive_path.display(),
            install_dir.display(),
            install_dir.display(),
            install_dir.join("ffmpegrust.exe").display(),
            install_dir.display(),
            archive_path.display(),
            install_dir.join("ffmpegrust.exe").display()
        );

        fs::write(&updater_script, script_content)
            .await
            .map_err(|e| anyhow!("Failed to create updater script: {}", e))?;

        // Launch the updater script and exit
        Command::new("cmd")
            .args(&["/C", &updater_script.to_string_lossy()])
            .spawn()
            .map_err(|e| anyhow!("Failed to launch updater: {}", e))?;

        // Schedule application exit
        std::process::exit(0);
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    async fn install_unix_update(
        &self,
        archive_path: &Path,
        install_dir: &std::path::Path,
    ) -> Result<()> {
        // Extract the archive
        let extract_dir = install_dir.join("update_temp");
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir).await.ok();
        }
        fs::create_dir(&extract_dir)
            .await
            .map_err(|e| anyhow!("Failed to create extract directory: {}", e))?;

        // Use tar to extract
        let output = Command::new("tar")
            .args([
                "-xzf",
                &archive_path.to_string_lossy(),
                "-C",
                &extract_dir.to_string_lossy(),
            ])
            .output()
            .map_err(|e| anyhow!("Failed to extract archive: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!(
                "Archive extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Find the extracted binary
        let extracted_binary = self.find_extracted_binary(&extract_dir).await?;
        let current_exe = std::env::current_exe()
            .map_err(|e| anyhow!("Failed to get current executable: {}", e))?;

        // Replace the current binary
        fs::copy(&extracted_binary, &current_exe)
            .await
            .map_err(|e| anyhow!("Failed to replace binary: {}", e))?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&current_exe)
                .await
                .map_err(|e| anyhow!("Failed to get metadata: {}", e))?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&current_exe, permissions)
                .await
                .map_err(|e| anyhow!("Failed to set permissions: {}", e))?;
        }

        // Cleanup
        fs::remove_dir_all(&extract_dir).await.ok();

        Ok(())
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    async fn find_extracted_binary(&self, extract_dir: &std::path::Path) -> Result<PathBuf> {
        let mut entries = fs::read_dir(extract_dir)
            .await
            .map_err(|e| anyhow!("Failed to read extract directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| anyhow!("Error reading entry: {}", e))?
        {
            let path = entry.path();
            if path.is_dir() {
                // Look for the binary in subdirectories
                let binary_path = path.join("ffmpegrust");
                if binary_path.exists() {
                    return Ok(binary_path);
                }
            } else if path.file_name().and_then(|n| n.to_str()) == Some("ffmpegrust") {
                return Ok(path);
            }
        }

        Err(anyhow!("Could not find extracted binary"))
    }

    pub fn should_check_for_updates(&self, last_check: Option<std::time::SystemTime>) -> bool {
        const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60); // 24 hours

        match last_check {
            Some(last) => {
                match last.elapsed() {
                    Ok(elapsed) => elapsed >= CHECK_INTERVAL,
                    Err(_) => true, // If we can't determine elapsed time, check anyway
                }
            }
            None => true, // Never checked before
        }
    }
}

// Helper function to get the current version from Cargo.toml
pub fn get_current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let updater = AutoUpdater::new("1.0.0").unwrap();
        assert_eq!(updater.current_version, Version::new(1, 0, 0));
    }

    #[test]
    fn test_platform_suffix() {
        let updater = AutoUpdater::new("1.0.0").unwrap();
        let suffix = updater.get_platform_suffix();
        assert!(!suffix.is_empty());
        assert!(
            suffix != "unknown"
                || cfg!(not(any(
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "linux"
                )))
        );
    }

    #[tokio::test]
    async fn test_update_check_interval() {
        let updater = AutoUpdater::new("1.0.0").unwrap();

        // Should check when never checked before
        assert!(updater.should_check_for_updates(None));

        // Should not check when just checked
        let now = std::time::SystemTime::now();
        assert!(!updater.should_check_for_updates(Some(now)));
    }
}
