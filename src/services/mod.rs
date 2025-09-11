use crate::conversion::ConversionTask;
use crate::events::{AppEvent, EventSender};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod config_service;
pub mod conversion_service;
pub mod file_service;
pub mod validation_service;

pub use config_service::ConfigService;
pub use conversion_service::ConversionService;
pub use file_service::FileService;
pub use validation_service::ValidationService;

#[derive(Clone)]
pub struct ServiceManager {
    pub conversion: ConversionService,
    pub file: FileService,
    pub config: ConfigService,
    pub validation: ValidationService,
    event_sender: EventSender,
}

impl ServiceManager {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            conversion: ConversionService::new(event_sender.clone()),
            file: FileService::new(event_sender.clone()),
            config: ConfigService::new(event_sender.clone()),
            validation: ValidationService::new(),
            event_sender,
        }
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize all services
        self.config.load_config().await?;
        self.file.initialize().await?;
        self.conversion.initialize().await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Gracefully shutdown all services
        self.conversion.shutdown().await?;
        self.config.save_config().await?;

        Ok(())
    }

    pub fn send_event(&self, event: AppEvent) {
        if let Err(e) = self.event_sender.send(event) {
            tracing::error!("Failed to send event: {}", e);
        }
    }
}

// Resource manager for tracking system resources
#[derive(Debug)]
pub struct ResourceManager {
    active_tasks: Arc<RwLock<HashMap<Uuid, ConversionTask>>>,
    max_concurrent: usize,
    temp_files: Arc<RwLock<Vec<PathBuf>>>,
    disk_space_threshold: u64,
}

impl ResourceManager {
    pub fn new(max_concurrent: usize, disk_space_threshold: u64) -> Self {
        Self {
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent,
            temp_files: Arc::new(RwLock::new(Vec::new())),
            disk_space_threshold,
        }
    }

    pub async fn can_start_conversion(&self) -> bool {
        let tasks = self.active_tasks.read().await;
        tasks.len() < self.max_concurrent
    }

    pub async fn add_task(&self, task_id: Uuid, task: ConversionTask) {
        let mut tasks = self.active_tasks.write().await;
        tasks.insert(task_id, task);
    }

    pub async fn remove_task(&self, task_id: &Uuid) -> Option<ConversionTask> {
        let mut tasks = self.active_tasks.write().await;
        tasks.remove(task_id)
    }

    pub async fn cancel_all_tasks(&self) {
        let tasks = self.active_tasks.read().await;
        for task in tasks.values() {
            task.cancel();
        }
    }

    pub async fn add_temp_file(&self, path: PathBuf) {
        let mut temp_files = self.temp_files.write().await;
        temp_files.push(path);
    }

    pub async fn cleanup_temp_files(&self) {
        let mut temp_files = self.temp_files.write().await;
        for path in temp_files.drain(..) {
            if path.exists() {
                if let Err(e) = tokio::fs::remove_file(&path).await {
                    tracing::warn!("Failed to remove temp file {:?}: {}", path, e);
                }
            }
        }
    }

    pub async fn check_disk_space(&self, output_path: &PathBuf) -> Result<bool, std::io::Error> {
        // Check available disk space
        if let Some(parent) = output_path.parent() {
            // This is a simplified check - in production you'd use a proper disk space check
            let _metadata = tokio::fs::metadata(parent).await?;
            // For now, just return true - implement proper disk space checking as needed
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// Service trait for common functionality
#[async_trait::async_trait]
pub trait Service {
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
}
