use crate::constants::{FFMPEG_TIMEOUT_SECONDS, MAX_CONCURRENT_CONVERSIONS};
use crate::conversion::{ConversionProgress, ConversionSettings, ConversionStatus, ConversionTask};
use crate::events::{AppEvent, EventSender};
use crate::services::{ResourceManager, Service};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ConversionServiceError {
    #[error("FFmpeg not found or not accessible")]
    FFmpegNotFound,
    #[error("Maximum concurrent conversions reached")]
    MaxConcurrentReached,
    #[error("Conversion task not found: {task_id}")]
    TaskNotFound { task_id: Uuid },
    #[error("Conversion already in progress: {task_id}")]
    AlreadyInProgress { task_id: Uuid },
    #[error("Invalid conversion settings: {message}")]
    InvalidSettings { message: String },
    #[error("Process error: {0}")]
    ProcessError(String),
    #[error("Timeout: conversion exceeded maximum duration")]
    Timeout,
    #[error("Cancelled by user")]
    Cancelled,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub enum ConversionTaskStatus {
    Queued,
    Running { progress: ConversionProgress },
    Completed { output_path: PathBuf },
    Failed { error: String },
    Cancelled,
}

#[derive(Debug)]
struct ManagedTask {
    task: ConversionTask,
    status: ConversionTaskStatus,
    created_at: std::time::Instant,
    settings: ConversionSettings,
    input_path: PathBuf,
    output_path: PathBuf,
}

#[derive(Clone)]
pub struct ConversionService {
    tasks: Arc<RwLock<HashMap<Uuid, ManagedTask>>>,
    resource_manager: Arc<ResourceManager>,
    concurrency_semaphore: Arc<Semaphore>,
    event_sender: EventSender,
}

impl ConversionService {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            resource_manager: Arc::new(ResourceManager::new(
                MAX_CONCURRENT_CONVERSIONS,
                crate::constants::DISK_SPACE_THRESHOLD_MB * 1024 * 1024, // Convert to bytes
            )),
            concurrency_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_CONVERSIONS)),
            event_sender,
        }
    }

    pub async fn start_conversion(
        &self,
        input_path: PathBuf,
        output_path: PathBuf,
        settings: ConversionSettings,
    ) -> Result<Uuid, ConversionServiceError> {
        // Check if we can start a new conversion
        if !self.resource_manager.can_start_conversion().await {
            return Err(ConversionServiceError::MaxConcurrentReached);
        }

        // Validate FFmpeg availability
        self.validate_ffmpeg().await?;

        // Create new task
        let task_id = Uuid::new_v4();
        let task = ConversionTask::new(
            input_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string(),
            settings.clone(),
        );

        // Validate the task
        task.validate()
            .map_err(|e| ConversionServiceError::InvalidSettings {
                message: e.user_message(),
            })?;

        let managed_task = ManagedTask {
            task,
            status: ConversionTaskStatus::Queued,
            created_at: std::time::Instant::now(),
            settings: settings.clone(),
            input_path: input_path.clone(),
            output_path: output_path.clone(),
        };

        // Add task to our tracking
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, managed_task);
        }

        // Send start event
        self.send_event(AppEvent::ConversionRequested {
            task_id,
            input: input_path,
            output: output_path,
            settings,
        });

        // Start the conversion in a background task
        let service_clone = self.clone();
        tokio::spawn(async move {
            service_clone.execute_conversion(task_id).await;
        });

        Ok(task_id)
    }

    async fn execute_conversion(&self, task_id: Uuid) {
        // Acquire semaphore permit for concurrency control
        let _permit = match self.concurrency_semaphore.acquire().await {
            Ok(permit) => permit,
            Err(_) => {
                self.handle_conversion_error(
                    task_id,
                    "Failed to acquire concurrency permit".to_string(),
                )
                .await;
                return;
            }
        };

        let (mut task, _settings, _input_path, output_path) = {
            let mut tasks = self.tasks.write().await;
            if let Some(managed_task) = tasks.get_mut(&task_id) {
                managed_task.status = ConversionTaskStatus::Running {
                    progress: ConversionProgress::default(),
                };
                (
                    ConversionTask::new_with_id(
                        task_id,
                        managed_task.input_path.to_string_lossy().to_string(),
                        managed_task.output_path.to_string_lossy().to_string(),
                        managed_task.settings.clone(),
                    ),
                    managed_task.settings.clone(),
                    managed_task.input_path.clone(),
                    managed_task.output_path.clone(),
                )
            } else {
                return;
            }
        };

        // Send conversion started event
        self.send_event(AppEvent::ConversionStarted(task_id));

        // Execute the conversion with timeout
        let conversion_future = self.run_conversion_with_progress(task_id, &mut task);
        let timeout_duration = Duration::from_secs(FFMPEG_TIMEOUT_SECONDS);

        match timeout(timeout_duration, conversion_future).await {
            Ok(Ok(())) => {
                // Conversion completed successfully
                self.handle_conversion_completion(task_id, output_path)
                    .await;
            }
            Ok(Err(error)) => {
                // Conversion failed
                self.handle_conversion_error(task_id, error).await;
            }
            Err(_) => {
                // Timeout occurred
                self.handle_conversion_timeout(task_id).await;
            }
        }

        // Remove from resource manager
        self.resource_manager.remove_task(&task_id).await;
    }

    async fn run_conversion_with_progress(
        &self,
        task_id: Uuid,
        task: &mut ConversionTask,
    ) -> Result<(), String> {
        // Execute the conversion and get the status receiver
        let mut status_receiver = task.execute().await.map_err(|e| e.user_message())?;

        // Add task to resource manager
        self.resource_manager.add_task(task_id, task.clone()).await;

        // Monitor progress
        while let Some(status) = status_receiver.recv().await {
            match status {
                ConversionStatus::Starting => {
                    // Already handled in execute_conversion
                }
                ConversionStatus::InProgress(progress) => {
                    // Update task status
                    {
                        let mut tasks = self.tasks.write().await;
                        if let Some(managed_task) = tasks.get_mut(&task_id) {
                            managed_task.status = ConversionTaskStatus::Running {
                                progress: progress.clone(),
                            };
                        }
                    }

                    // Send progress event
                    self.send_event(AppEvent::ConversionProgress { task_id, progress });
                }
                ConversionStatus::Completed => {
                    return Ok(());
                }
                ConversionStatus::Failed(error) => {
                    return Err(error);
                }
                ConversionStatus::Cancelled => {
                    return Err("Conversion was cancelled".to_string());
                }
            }
        }

        Err("Conversion process ended unexpectedly".to_string())
    }

    async fn handle_conversion_completion(&self, task_id: Uuid, output_path: PathBuf) {
        {
            let mut tasks = self.tasks.write().await;
            if let Some(managed_task) = tasks.get_mut(&task_id) {
                managed_task.status = ConversionTaskStatus::Completed {
                    output_path: output_path.clone(),
                };
            }
        }

        self.send_event(AppEvent::ConversionCompleted(task_id));
        tracing::info!("Conversion {} completed successfully", task_id);
    }

    async fn handle_conversion_error(&self, task_id: Uuid, error: String) {
        {
            let mut tasks = self.tasks.write().await;
            if let Some(managed_task) = tasks.get_mut(&task_id) {
                managed_task.status = ConversionTaskStatus::Failed {
                    error: error.clone(),
                };
            }
        }

        self.send_event(AppEvent::ConversionFailed {
            task_id,
            error: error.clone(),
        });
        tracing::error!("Conversion {} failed: {}", task_id, error);
    }

    async fn handle_conversion_timeout(&self, task_id: Uuid) {
        // Cancel the task first
        if let Some(task) = self.resource_manager.remove_task(&task_id).await {
            task.cancel();
        }

        {
            let mut tasks = self.tasks.write().await;
            if let Some(managed_task) = tasks.get_mut(&task_id) {
                managed_task.status = ConversionTaskStatus::Failed {
                    error: "Conversion timed out".to_string(),
                };
            }
        }

        self.send_event(AppEvent::ConversionFailed {
            task_id,
            error: "Conversion exceeded maximum duration".to_string(),
        });
        tracing::error!("Conversion {} timed out", task_id);
    }

    pub async fn cancel_conversion(&self, task_id: Uuid) -> Result<(), ConversionServiceError> {
        // Remove and cancel the task
        if let Some(task) = self.resource_manager.remove_task(&task_id).await {
            task.cancel();
        }

        // Update task status
        {
            let mut tasks = self.tasks.write().await;
            if let Some(managed_task) = tasks.get_mut(&task_id) {
                managed_task.status = ConversionTaskStatus::Cancelled;
            } else {
                return Err(ConversionServiceError::TaskNotFound { task_id });
            }
        }

        self.send_event(AppEvent::ConversionCancelled(()));
        tracing::info!("Conversion {} cancelled", task_id);
        Ok(())
    }

    pub async fn get_task_status(&self, task_id: Uuid) -> Option<ConversionTaskStatus> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).map(|task| task.status.clone())
    }

    pub async fn get_all_tasks(&self) -> Vec<(Uuid, ConversionTaskStatus)> {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .map(|(id, task)| (*id, task.status.clone()))
            .collect()
    }

    pub async fn get_active_task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|task| matches!(task.status, ConversionTaskStatus::Running { .. }))
            .count()
    }

    pub async fn cancel_all_conversions(&self) -> Result<(), ConversionServiceError> {
        // Get all active task IDs
        let active_tasks: Vec<Uuid> = {
            let tasks = self.tasks.read().await;
            tasks
                .iter()
                .filter_map(|(id, task)| {
                    if matches!(
                        task.status,
                        ConversionTaskStatus::Running { .. } | ConversionTaskStatus::Queued
                    ) {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Cancel each active task
        for task_id in active_tasks {
            let _ = self.cancel_conversion(task_id).await; // Continue even if individual cancellation fails
        }

        // Cancel all tasks in resource manager
        self.resource_manager.cancel_all_tasks().await;

        tracing::info!("All conversions cancelled");
        Ok(())
    }

    pub async fn cleanup_completed_tasks(&self) -> usize {
        let mut tasks = self.tasks.write().await;
        let initial_count = tasks.len();

        tasks.retain(|_, task| {
            !matches!(
                task.status,
                ConversionTaskStatus::Completed { .. }
                    | ConversionTaskStatus::Failed { .. }
                    | ConversionTaskStatus::Cancelled
            )
        });

        let removed_count = initial_count - tasks.len();
        if removed_count > 0 {
            tracing::info!("Cleaned up {} completed tasks", removed_count);
        }
        removed_count
    }

    async fn validate_ffmpeg(&self) -> Result<(), ConversionServiceError> {
        match tokio::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(ConversionServiceError::FFmpegNotFound)
                }
            }
            Err(_) => Err(ConversionServiceError::FFmpegNotFound),
        }
    }

    fn send_event(&self, event: AppEvent) {
        if let Err(e) = self.event_sender.send(event) {
            tracing::error!("Failed to send conversion event: {}", e);
        }
    }

    pub async fn get_conversion_statistics(&self) -> ConversionStatistics {
        let tasks = self.tasks.read().await;
        let mut stats = ConversionStatistics::default();

        for task in tasks.values() {
            match &task.status {
                ConversionTaskStatus::Queued => stats.queued += 1,
                ConversionTaskStatus::Running { .. } => stats.running += 1,
                ConversionTaskStatus::Completed { .. } => {
                    stats.completed += 1;
                    stats.total_conversion_time += task.created_at.elapsed();
                }
                ConversionTaskStatus::Failed { .. } => stats.failed += 1,
                ConversionTaskStatus::Cancelled => stats.cancelled += 1,
            }
        }

        stats.total = tasks.len();
        stats
    }
}

#[async_trait::async_trait]
impl Service for ConversionService {
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate FFmpeg is available
        self.validate_ffmpeg().await?;
        tracing::info!("Conversion service initialized");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Cancel all active conversions
        self.cancel_all_conversions().await?;

        // Cleanup resources
        self.resource_manager.cleanup_temp_files().await;

        tracing::info!("Conversion service shutdown");
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConversionStatistics {
    pub total: usize,
    pub queued: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub total_conversion_time: Duration,
}

impl ConversionStatistics {
    pub fn average_conversion_time(&self) -> Option<Duration> {
        if self.completed > 0 {
            Some(self.total_conversion_time / self.completed as u32)
        } else {
            None
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total > 0 {
            self.completed as f64 / self.total as f64
        } else {
            0.0
        }
    }
}
