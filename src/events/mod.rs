use crate::conversion::ConversionProgress;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AppEvent {
    // File operations (handled directly in UI)

    // Conversion events
    ConversionRequested {
        task_id: Uuid,
        input: PathBuf,
        output: PathBuf,
        settings: crate::conversion::ConversionSettings,
    },
    ConversionStarted(Uuid),
    ConversionProgress {
        task_id: Uuid,
        progress: ConversionProgress,
    },
    ConversionCompleted(Uuid),
    ConversionFailed {
        task_id: Uuid,
        error: String,
    },
    ConversionCancelled(()),

    // UI events
    TabChanged(crate::app::ActiveTab),
    PresetApplied(String),
    SettingsChanged,

    // Config events
    ConfigLoaded,
    ConfigSaved,

    // Error events
    ErrorOccurred(String),
    ErrorCleared,
}

pub type EventSender = tokio::sync::mpsc::UnboundedSender<AppEvent>;
pub type EventReceiver = tokio::sync::mpsc::UnboundedReceiver<AppEvent>;

pub fn create_event_channel() -> (EventSender, EventReceiver) {
    tokio::sync::mpsc::unbounded_channel()
}
