# FFmpeg Rust

A simple, minimalistic GUI for video conversion and remuxing using FFmpeg.

## Features

- **File Selection**: Easy input file selection with folder memory
- **Output Management**: Choose output folder with automatic path remembering
- **Conversion Modes**:
  - **Convert**: Full video/audio conversion with codec selection
  - **Remux**: Container format change without re-encoding
- **Format Support**: MP4, MKV, MOV, AVI, WebM
- **Video Codecs**: H.264, H.265, VP9, Copy
- **Audio Codecs**: AAC, MP3, FLAC, PCM (16-bit), Copy
- **Real-time Progress**: Live progress bar with percentage and time estimation
- **Custom Presets**: Save and load your favorite conversion settings
- **Help System**: Check FFmpeg installation, updates, and about information

## Requirements

- **FFmpeg**: Must be installed and available in your system PATH
- **Operating System**: Windows, macOS, or Linux

## Installation

### From Binary (Recommended)

1. Download the latest release for your platform from the [Releases](https://github.com/yourusername/ffmpegrust/releases) page
2. Extract the archive
3. Run the executable

### From Source

1. Install [Rust](https://rustup.rs/) if you haven't already
2. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/ffmpegrust.git
   cd ffmpegrust
   ```
3. Build and run:
   ```bash
   cargo build --release
   cargo run
   ```

## FFmpeg Installation

### Windows
- Download from [FFmpeg.org](https://ffmpeg.org/download.html)
- Extract and add to your PATH environment variable

### macOS
```bash
brew install ffmpeg
```

### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install ffmpeg
```

### Linux (Fedora)
```bash
sudo dnf install ffmpeg
```

## Usage

1. **Select Input File**: Click "Select Input File" and choose your video
2. **Choose Output Folder**: Click "Select Output Folder" for conversion destination
3. **Set Mode**: Choose between "Convert" or "Remux"
4. **Configure Settings** (Convert mode only):
   - Select output format (MP4, MKV, MOV, etc.)
   - Choose video codec (H.264, H.265, VP9, or Copy)
   - Choose audio codec (AAC, MP3, FLAC, PCM, or Copy)
   - Optionally set advanced settings (bitrates, resolution, frame rate)
5. **Start Conversion**: Click "Start Conversion" and monitor progress
6. **Save Presets**: Save frequently used settings as custom presets

## Presets

Create and manage custom presets to quickly apply your favorite conversion settings:

- Click "Save Current" to save the current configuration
- Use the preset dropdown to load saved presets
- Delete unwanted presets with the "Delete" button

## Configuration

The application automatically saves:
- Last used input and output folders
- Window size and position
- Auto-update preferences

Configuration files are stored in:
- **Windows**: `%APPDATA%\ffmpegrust\`
- **macOS**: `~/Library/Application Support/ffmpegrust/`
- **Linux**: `~/.config/ffmpegrust/`

## Help & Updates

Access the Help menu to:
- **Check FFmpeg**: Verify FFmpeg installation and version
- **Check for Updates**: Manually check for application updates
- **About**: View application information and links

## Building

### Dependencies
- Rust 1.70 or later
- FFmpeg (for runtime functionality)

### Build Commands
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run directly
cargo run

# Run tests
cargo test
```

## Project Structure

```
ffmpegrust/
├── src/
│   ├── main.rs          # Application entry point
│   ├── app.rs           # Main UI and application logic
│   ├── config.rs        # Configuration management
│   ├── conversion.rs    # FFmpeg conversion logic
│   ├── presets.rs       # Preset management
│   ├── updater.rs       # Auto-update functionality
│   └── utils.rs         # Utility functions
├── assets/
│   └── README.md        # Icon requirements and guidelines
├── Cargo.toml           # Rust dependencies
└── README.md           # This file
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [FFmpeg](https://ffmpeg.org/) - The multimedia framework that powers conversions
- [egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [Rust](https://www.rust-lang.org/) - The programming language

## Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/yourusername/ffmpegrust/issues) page
2. Ensure FFmpeg is properly installed
3. Create a new issue with detailed information about your problem

---

**Note**: This application is a frontend for FFmpeg. All conversion quality and capabilities depend on your FFmpeg installation.