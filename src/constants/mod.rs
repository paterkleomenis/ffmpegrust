// UI constants for future interface improvements
#[allow(dead_code)]
pub const UI_UPDATE_INTERVAL_MS: u64 = 250;
#[allow(dead_code)]
pub const DEFAULT_WINDOW_SIZE: [f32; 2] = [900.0, 700.0];
#[allow(dead_code)]
pub const MIN_WINDOW_SIZE: [f32; 2] = [800.0, 600.0];
#[allow(dead_code)]
pub const PROGRESS_BUFFER_SIZE: usize = 1024;

// UI Styling Constants for future themes
#[allow(dead_code)]
pub const HEADER_HEIGHT: f32 = 80.0;
#[allow(dead_code)]
pub const CARD_PADDING: f32 = 20.0;
#[allow(dead_code)]
pub const CARD_SPACING: f32 = 15.0;
#[allow(dead_code)]
pub const CARD_ROUNDING: f32 = 10.0;
#[allow(dead_code)]
pub const BUTTON_PADDING: [f32; 2] = [12.0, 8.0];
#[allow(dead_code)]
pub const TAB_BUTTON_PADDING: [f32; 2] = [25.0, 15.0];
#[allow(dead_code)]
pub const ITEM_SPACING: [f32; 2] = [10.0, 10.0];

// Color constants for future themes
#[allow(dead_code)]
pub const WINDOW_BG_COLOR: u8 = 20;
#[allow(dead_code)]
pub const PANEL_BG_COLOR: u8 = 25;
#[allow(dead_code)]
pub const CARD_BG_COLOR: u8 = 30;
#[allow(dead_code)]
pub const CARD_BORDER_COLOR: u8 = 45;
#[allow(dead_code)]
pub const HEADER_BG_COLOR: u8 = 15;

// Codec options for future preset system
#[allow(dead_code)]
pub const DEFAULT_CODECS: &[(&str, &str)] = &[
    ("libx264", "H.264 (x264)"),
    ("libx265", "H.265/HEVC (x265)"),
    ("libvpx-vp9", "VP9"),
    ("libaom-av1", "AV1"),
    ("copy", "Copy (No Re-encoding)"),
];

#[allow(dead_code)]
pub const AUDIO_CODECS: &[(&str, &str)] = &[
    ("aac", "AAC"),
    ("libmp3lame", "MP3"),
    ("libopus", "Opus"),
    ("flac", "FLAC"),
    ("pcm_s16le", "PCM 16-bit (Little Endian)"),
    ("pcm_s24le", "PCM 24-bit (Little Endian)"),
    ("pcm_s32le", "PCM 32-bit (Little Endian)"),
    ("pcm_f32le", "PCM 32-bit Float"),
    ("pcm_f64le", "PCM 64-bit Float"),
    ("pcm_s16be", "PCM 16-bit (Big Endian)"),
    ("pcm_s24be", "PCM 24-bit (Big Endian)"),
    ("pcm_u8", "PCM 8-bit Unsigned"),
    ("copy", "Copy (No Re-encoding)"),
];

#[allow(dead_code)]
pub const CONTAINER_FORMATS: &[(&str, &str)] = &[
    ("mp4", "MP4"),
    ("mkv", "MKV"),
    ("avi", "AVI"),
    ("mov", "MOV"),
    ("wav", "WAV"),
    ("webm", "WebM"),
    ("flv", "FLV"),
    ("wmv", "WMV"),
];

#[allow(dead_code)]
pub const QUALITY_PRESETS: &[(&str, &str)] = &[
    ("18", "Lossless (18)"),
    ("20", "Very High (20)"),
    ("23", "High (23)"),
    ("26", "Medium (26)"),
    ("30", "Low (30)"),
    ("35", "Very Low (35)"),
];

// FFmpeg process constants for future timeout handling
#[allow(dead_code)]
pub const FFMPEG_TIMEOUT_SECONDS: u64 = 3600; // 1 hour max
#[allow(dead_code)]
pub const PROGRESS_UPDATE_INTERVAL_MS: u64 = 100;
#[allow(dead_code)]
pub const CANCELLATION_CHECK_INTERVAL_MS: u64 = 100;

// File handling constants for validation
#[allow(dead_code)]
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "avi", "mkv", "mov", "wav", "wmv", "flv", "webm", "m4v", "3gp", "ts", "mts", "m2ts",
    "vob", "ogv",
];

// System resource constants
#[allow(dead_code)]
pub const MAX_CONCURRENT_CONVERSIONS: usize = 2;
#[allow(dead_code)]
pub const DISK_SPACE_THRESHOLD_MB: u64 = 1024; // 1GB minimum free space
#[allow(dead_code)]
pub const MAX_LOG_LINES: usize = 1000;

// Format compatibility for smart remuxing
#[allow(dead_code)]
pub const REMUX_COMPATIBLE_FORMATS: &[(&str, &[&str])] = &[
    ("mp4", &["mov", "mkv", "avi"]),
    ("mov", &["mp4", "mkv", "avi"]),
    ("mkv", &["mp4", "mov", "avi", "webm"]),
    ("avi", &["mp4", "mov", "mkv"]),
    ("webm", &["mkv"]),
    ("flv", &["mp4", "mov"]),
];

// Container codec recommendations for presets
#[allow(dead_code)]
pub const CONTAINER_CODEC_RECOMMENDATIONS: &[(&str, &[&str], &[&str])] = &[
    (
        "mp4",
        &["libx264", "libx265", "copy"],
        &["aac", "mp3", "copy"],
    ),
    (
        "mov",
        &["libx264", "libx265", "prores_ks", "copy"],
        &["aac", "pcm_s16le", "pcm_s24le", "pcm_s32le", "copy"],
    ),
    (
        "mkv",
        &["libx264", "libx265", "libvpx-vp9", "copy"],
        &["aac", "opus", "flac", "pcm_s16le", "pcm_s24le", "copy"],
    ),
    (
        "wav",
        &["copy"],
        &["pcm_s16le", "pcm_s24le", "pcm_s32le", "pcm_f32le", "copy"],
    ),
    ("webm", &["libvpx-vp9", "copy"], &["libopus", "copy"]),
    ("avi", &["libx264", "copy"], &["mp3", "pcm_s16le", "copy"]),
];

// Quick remux presets for future feature
#[allow(dead_code)]
pub const REMUX_PRESETS: &[(&str, &str)] = &[
    ("Fast MP4", "mp4"),
    ("Fast MOV", "mov"),
    ("Fast MKV", "mkv"),
    ("Fast WebM", "webm"),
    ("Fast AVI", "avi"),
];
