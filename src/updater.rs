use futures_util::StreamExt;
use reqwest;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
    pub published_at: String,
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    CheckingForUpdates,
    UpdateAvailable(UpdateInfo),
    NoUpdateAvailable,
    DownloadingUpdate(f32), // percentage
    InstallingUpdate,
    Error(String),
}

#[derive(Clone)]
pub struct Updater {
    current_version: Version,
    github_repo: String,
    client: reqwest::Client,
}

impl Updater {
    pub fn new(current_version: &str, github_repo: &str) -> Result<Self, String> {
        let current_version = Version::parse(current_version)
            .map_err(|e| format!("Invalid current version: {}", e))?;

        let client = reqwest::Client::builder()
            .user_agent("FFmpegRust-Updater/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            current_version,
            github_repo: github_repo.to_string(),
            client,
        })
    }

    pub async fn check_for_updates(&self) -> UpdateStatus {
        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.github_repo
        );

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<GitHubRelease>().await {
                        Ok(release) => self.process_release(release),
                        Err(e) => {
                            UpdateStatus::Error(format!("Failed to parse release info: {}", e))
                        }
                    }
                } else {
                    UpdateStatus::Error(format!("GitHub API request failed: {}", response.status()))
                }
            }
            Err(e) => UpdateStatus::Error(format!("Network error: {}", e)),
        }
    }

    fn process_release(&self, release: GitHubRelease) -> UpdateStatus {
        // Parse the version from tag_name (remove 'v' prefix if present)
        let version_str = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name);

        match Version::parse(version_str) {
            Ok(remote_version) => {
                if remote_version > self.current_version {
                    // Find the appropriate download URL
                    if let Some(download_url) = self.find_download_url(&release.assets) {
                        let update_info = UpdateInfo {
                            version: remote_version.to_string(),
                            download_url,
                            release_notes: release.body.unwrap_or_default(),
                            published_at: release.published_at.unwrap_or_default(),
                        };
                        UpdateStatus::UpdateAvailable(update_info)
                    } else {
                        UpdateStatus::Error("No compatible download found".to_string())
                    }
                } else {
                    UpdateStatus::NoUpdateAvailable
                }
            }
            Err(e) => UpdateStatus::Error(format!("Invalid remote version: {}", e)),
        }
    }

    fn find_download_url(&self, assets: &[GitHubAsset]) -> Option<String> {
        // Look for platform-specific executable
        let platform_suffix = if cfg!(target_os = "windows") {
            ".exe"
        } else if cfg!(target_os = "macos") {
            "-macos"
        } else {
            "-linux"
        };

        // Try to find a platform-specific asset
        for asset in assets {
            if asset.name.contains(platform_suffix) {
                return Some(asset.browser_download_url.clone());
            }
        }

        // Fallback to any executable file
        for asset in assets {
            if asset.name.ends_with(".exe") || !asset.name.contains('.') {
                return Some(asset.browser_download_url.clone());
            }
        }

        None
    }

    pub async fn download_update(
        &self,
        update_info: &UpdateInfo,
        sender: Option<tokio::sync::mpsc::UnboundedSender<f32>>,
    ) -> Result<PathBuf, String> {
        let response = self
            .client
            .get(&update_info.download_url)
            .send()
            .await
            .map_err(|e| format!("Failed to start download: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);

        // Get the file name from the URL
        let file_name = update_info
            .download_url
            .split('/')
            .last()
            .unwrap_or("ffmpegrust_update");

        // Create temporary directory for download
        let temp_dir = std::env::temp_dir().join("ffmpegrust_updates");
        if let Err(e) = fs::create_dir_all(&temp_dir).await {
            return Err(format!("Failed to create temp directory: {}", e));
        }

        let file_path = temp_dir.join(file_name);

        // Download the file with progress reporting
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| format!("Failed to create download file: {}", e))?;

        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Failed to read chunk: {}", e))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write chunk: {}", e))?;

            downloaded += chunk.len() as u64;

            // Report progress
            if let Some(ref sender) = sender {
                if total_size > 0 {
                    let progress = (downloaded as f32 / total_size as f32) * 100.0;
                    let _ = sender.send(progress);
                }
            }
        }

        file.flush()
            .await
            .map_err(|e| format!("Failed to flush download file: {}", e))?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file
                .metadata()
                .await
                .map_err(|e| format!("Failed to get file metadata: {}", e))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&file_path, perms)
                .await
                .map_err(|e| format!("Failed to set file permissions: {}", e))?;
        }

        Ok(file_path)
    }

    pub async fn apply_update(&self, update_file: &PathBuf) -> Result<(), String> {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;

        #[cfg(windows)]
        {
            // On Windows, we need to rename the current exe and replace it
            let backup_path = current_exe.with_extension("exe.old");

            fs::rename(&current_exe, &backup_path)
                .await
                .map_err(|e| format!("Failed to backup current executable: {}", e))?;

            fs::copy(update_file, &current_exe)
                .await
                .map_err(|e| format!("Failed to replace executable: {}", e))?;
        }

        #[cfg(not(windows))]
        {
            fs::copy(update_file, &current_exe)
                .await
                .map_err(|e| format!("Failed to replace executable: {}", e))?;
        }

        Ok(())
    }

    pub async fn restart_application(&self) -> Result<(), String> {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;

        // Give a moment for the current process to finish cleanly
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        #[cfg(windows)]
        {
            std::process::Command::new("cmd")
                .args(&[
                    "/C",
                    "timeout",
                    "1",
                    ">nul",
                    "&&",
                    &current_exe.to_string_lossy(),
                ])
                .spawn()
                .map_err(|e| format!("Failed to restart application: {}", e))?;
        }

        #[cfg(not(windows))]
        {
            std::process::Command::new("sh")
                .args(&[
                    "-c",
                    &format!("sleep 1 && exec '{}'", current_exe.display()),
                ])
                .spawn()
                .map_err(|e| format!("Failed to restart application: {}", e))?;
        }

        std::process::exit(0);
    }
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}
