use crate::conversion::{ConversionProgress, ConversionSettings};
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle,
    Converting {
        task_id: Uuid,
        start_time: Instant,
        progress: ConversionProgress,
    },
    Completed {
        task_id: Uuid,
        output_path: PathBuf,
        duration: std::time::Duration,
    },
    Failed {
        task_id: Uuid,
        error: String,
    },
    Cancelled {
        task_id: Uuid,
    },
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Idle
    }
}

impl AppState {
    pub fn is_idle(&self) -> bool {
        matches!(self, AppState::Idle)
    }

    pub fn is_converting(&self) -> bool {
        matches!(self, AppState::Converting { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            AppState::Completed { .. } | AppState::Failed { .. } | AppState::Cancelled { .. }
        )
    }

    pub fn get_task_id(&self) -> Option<Uuid> {
        match self {
            AppState::Converting { task_id, .. }
            | AppState::Completed { task_id, .. }
            | AppState::Failed { task_id, .. }
            | AppState::Cancelled { task_id, .. } => Some(*task_id),
            AppState::Idle => None,
        }
    }

    pub fn get_progress(&self) -> Option<&ConversionProgress> {
        match self {
            AppState::Converting { progress, .. } => Some(progress),
            _ => None,
        }
    }

    pub fn get_error(&self) -> Option<&str> {
        match self {
            AppState::Failed { error, .. } => Some(error),
            _ => None,
        }
    }

    pub fn transition_to_converting(task_id: Uuid) -> Self {
        AppState::Converting {
            task_id,
            start_time: Instant::now(),
            progress: ConversionProgress::default(),
        }
    }

    pub fn update_progress(&mut self, new_progress: ConversionProgress) {
        if let AppState::Converting { progress, .. } = self {
            *progress = new_progress;
        }
    }

    pub fn transition_to_completed(self, output_path: PathBuf) -> Self {
        match self {
            AppState::Converting {
                task_id,
                start_time,
                ..
            } => AppState::Completed {
                task_id,
                output_path,
                duration: start_time.elapsed(),
            },
            _ => self,
        }
    }

    pub fn transition_to_failed(self, error: String) -> Self {
        match self {
            AppState::Converting { task_id, .. } => AppState::Failed { task_id, error },
            _ => self,
        }
    }

    pub fn transition_to_cancelled(self) -> Self {
        match self {
            AppState::Converting { task_id, .. } => AppState::Cancelled { task_id },
            _ => self,
        }
    }

    pub fn reset_to_idle(&mut self) {
        *self = AppState::Idle;
    }
}

#[derive(Debug, Clone)]
pub struct AppData {
    pub input_file: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub settings: ConversionSettings,
    pub state: AppState,
    pub error_message: Option<String>,
    pub progress_changed: bool,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            input_file: None,
            output_file: None,
            settings: ConversionSettings::default(),
            state: AppState::default(),
            error_message: None,
            progress_changed: false,
        }
    }
}

impl AppData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_input_file(&mut self, path: PathBuf) {
        self.input_file = Some(path);
        self.clear_error();
    }

    pub fn set_output_file(&mut self, path: PathBuf) {
        self.output_file = Some(path);
    }

    pub fn update_settings(&mut self, settings: ConversionSettings) {
        self.settings = settings;
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn has_error(&self) -> bool {
        self.error_message.is_some()
    }

    pub fn can_start_conversion(&self) -> bool {
        self.input_file.is_some()
            && self.output_file.is_some()
            && self.state.is_idle()
            && !self.has_error()
    }

    pub fn start_conversion(&mut self, task_id: Uuid) {
        if self.can_start_conversion() {
            self.state = AppState::transition_to_converting(task_id);
            self.clear_error();
        }
    }

    pub fn update_conversion_progress(&mut self, progress: ConversionProgress) {
        self.state.update_progress(progress);
        self.progress_changed = true;
    }

    pub fn complete_conversion(&mut self, output_path: PathBuf) {
        let old_state = std::mem::take(&mut self.state);
        self.state = old_state.transition_to_completed(output_path);
    }

    pub fn fail_conversion(&mut self, error: String) {
        let old_state = std::mem::take(&mut self.state);
        self.state = old_state.transition_to_failed(error.clone());
        self.set_error(error);
    }

    pub fn cancel_conversion(&mut self) {
        let old_state = std::mem::take(&mut self.state);
        self.state = old_state.transition_to_cancelled();
    }

    pub fn reset_conversion(&mut self) {
        self.state.reset_to_idle();
        self.clear_error();
        self.progress_changed = false;
    }

    pub fn mark_progress_handled(&mut self) {
        self.progress_changed = false;
    }
}
