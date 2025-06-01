# FFmpeg Converter Pro

A modern, professional GUI application for video conversion and remuxing built with Rust. Features real-time progress tracking, hardware acceleration, and an intuitive tabbed interface.

### 🚀 Performance
- **Hardware Acceleration**: NVIDIA NVENC, Intel QSV, AMD VCE support
- **Multi-threaded**: Efficient CPU utilization with async processing
- **Real-time Monitoring**: Frame count, FPS, speed, ETA tracking
- **Fast Remuxing**: Container changes without re-encoding (20-50x speed)

### 📽️ Format Support
- **Containers**: MP4, MKV, MOV, AVI, WebM, FLV, WMV, M4V
- **Video Codecs**: H.264, H.265, VP8, VP9, AV1, ProRes
- **Audio Codecs**: AAC, MP3, Opus, Vorbis, FLAC, PCM
- **Quality Control**: CRF 18-30 range with visual indicators

### 🎯 Quick Presets
- **🌐 Web Optimized**: H.264 + AAC for streaming platforms
- **💎 High Quality**: H.265 + AAC for archival storage
- **📱 Small File**: Aggressive compression for limited storage
- **⚡ Fast Remux**: Instant container changes
- **🎵 Pro Audio**: Smart copy with PCM audio conversion

## 🚀 Installation

### Prerequisites

**FFmpeg Installation:**

```bash
# Windows (using winget)
winget install Gyan.FFmpeg

# macOS (using Homebrew)
brew install ffmpeg

# Linux (Ubuntu/Debian)
sudo apt update && sudo apt install ffmpeg

# Linux (Fedora/RHEL)
sudo dnf install ffmpeg
```

**Rust Installation:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/paterkleomenis/ffmpegrust.git
cd ffmpegrust

# Build and run
cargo build --release
./target/release/ffmpegrust

# Or use the convenience scripts
# Linux/macOS
./run.sh

# Windows
run.bat
```

## 🎯 Usage

### Basic Workflow

1. **📁 Select Files**: Browse for input video, set output location
2. **🎯 Choose Preset**: Pick from optimized presets or go custom
3. **⚙️ Adjust Settings**: Fine-tune codecs and quality (Advanced tab)
4. **🚀 Start Conversion**: Monitor progress in real-time
5. **✅ Complete**: Automatic status updates and notifications

### Common Use Cases

#### Web/Social Media Content
```
Preset: Web Optimized
Result: Universal compatibility, ~50% file size
Best for: YouTube, social platforms, streaming
```

#### Archive & Storage
```
Preset: High Quality
Result: Excellent quality, ~60% file size
Best for: Long-term storage, future-proofing
```

#### Quick Format Changes
```
Mode: Fast Remux
Result: Identical quality, 20-50x speed
Best for: Container changes without re-encoding
```

#### Professional Audio
```
Preset: Pro Audio (MOV PCM)
Result: Video copy + high-quality audio
Best for: Audio post-production workflows
```

### Quality vs Speed
| CRF | Quality | File Size | Use Case |
|-----|---------|-----------|----------|
| 18-20 | ✨ Visually lossless | 70% | Professional |
| 21-23 | 🎯 High quality | 50% | General use |
| 24-26 | 👌 Good quality | 40% | Web content |
| 27-28 | 📱 Acceptable | 32% | Mobile/limited bandwidth |

## 🛠️ Configuration

### Settings Persistence
Application settings are automatically saved to:
- **Windows**: `%APPDATA%\ffmpegrust\config.json`
- **macOS**: `~/Library/Application Support/ffmpegrust/config.json`
- **Linux**: `~/.config/ffmpegrust/config.json`

### Hardware Acceleration Setup

**NVIDIA (Windows/Linux):**
```bash
# Verify NVENC support
ffmpeg -encoders | grep nvenc
# Should show: h264_nvenc, hevc_nvenc
```

**Intel QSV (Linux):**
```bash
# Install Intel media driver
sudo apt install intel-media-driver
# Verify QSV support
ffmpeg -encoders | grep qsv
```

**Slow conversion speeds**
- ✅ Enable hardware acceleration in Performance tab
- ✅ Close other resource-intensive applications
- ✅ Use faster codecs (H.264 vs H.265)
- ✅ Lower quality settings if acceptable

**Audio sync issues**
- ✅ Try different audio codec
- ✅ Use "copy" for audio if only changing video
- ✅ Check source file integrity


### Dependencies
```toml
eframe = "0.24"      # GUI framework
egui = "0.24"        # Immediate mode GUI
rfd = "0.12"         # Native file dialogs
regex = "1.0"        # Progress parsing
serde = "1.0"        # Settings serialization
dirs = "5.0"         # System directories
anyhow = "1.0"       # Error handling
```
