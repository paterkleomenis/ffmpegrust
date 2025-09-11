// App Constants
pub const APP_NAME: &str = "FFmpeg Converter Pro";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// FFmpeg Constants
pub const FFMPEG_TIMEOUT_SECONDS: u64 = 3600; // 1 hour max
pub const PROGRESS_UPDATE_INTERVAL_MS: u64 = 100;
pub const CANCELLATION_CHECK_INTERVAL_MS: u64 = 100;

// File handling
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ogv",
];

// System limits
pub const MAX_CONCURRENT_CONVERSIONS: usize = 2;
pub const DISK_SPACE_THRESHOLD_MB: u64 = 1024; // 1GB minimum free space
pub const MAX_LOG_LINES: usize = 1000;

// Remux compatibility - format -> compatible input formats
pub const REMUX_COMPATIBLE_FORMATS: &[(&str, &[&str])] = &[
    ("mp4", &["h264", "h265", "hevc", "aac", "mp3"]),
    (
        "mkv",
        &[
            "h264", "h265", "hevc", "vp8", "vp9", "av1", "aac", "mp3", "flac", "opus",
        ],
    ),
    ("webm", &["vp8", "vp9", "av1", "opus", "vorbis"]),
    ("avi", &["h264", "xvid", "divx", "mp3", "aac"]),
    ("mov", &["h264", "h265", "hevc", "prores", "aac", "mp3"]),
];

// Container + Video Codec + Audio Codec recommendations
pub const CONTAINER_CODEC_RECOMMENDATIONS: &[(&str, &[&str], &[&str])] = &[
    // Container, Video Codecs, Audio Codecs
    ("mp4", &["libx264", "libx265"], &["aac", "mp3"]),
    (
        "mkv",
        &["libx264", "libx265", "libvpx-vp9", "libaom-av1"],
        &["aac", "mp3", "flac", "libopus"],
    ),
    (
        "webm",
        &["libvpx-vp8", "libvpx-vp9", "libaom-av1"],
        &["libopus", "libvorbis"],
    ),
    ("avi", &["libx264", "libxvid"], &["mp3", "aac"]),
    ("mov", &["libx264", "libx265", "prores"], &["aac", "mp3"]),
    ("flv", &["libx264"], &["aac", "mp3"]),
    ("3gp", &["libx264"], &["aac"]),
    ("wmv", &["wmv2"], &["wmav2"]),
    ("ogv", &["libtheora"], &["libvorbis"]),
    ("m4v", &["libx264"], &["aac"]),
];

// Quick remux presets
pub const REMUX_PRESETS: &[(&str, &str)] = &[
    ("MP4 Fast", "Fast remux to MP4 container"),
    ("MKV Universal", "Remux to MKV for maximum compatibility"),
    ("WebM Web", "Remux to WebM for web use"),
    ("MOV Apple", "Remux to MOV for Apple devices"),
];
