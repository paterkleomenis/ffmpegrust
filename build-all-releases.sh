#!/bin/bash

# FFmpeg Converter Pro - Complete Release Builder
# Builds Windows installer and Mac/Linux executables

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 FFmpeg Converter Pro - Complete Release Builder${NC}"
echo ""

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | cut -d '"' -f 2)
echo -e "${BLUE}Building version: ${GREEN}v${VERSION}${NC}"
echo ""

# Create release directory
RELEASE_DIR="releases/v${VERSION}"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Function to build for a specific target
build_target() {
    local target=$1
    local platform_name=$2
    local description=$3

    echo -e "${BLUE}📦 Building $description...${NC}"

    # Add target if not present
    rustup target add "$target" 2>/dev/null || true

    # Build
    if cargo build --release --target "$target"; then
        echo -e "${GREEN}✅ Build successful for $description${NC}"
        return 0
    else
        echo -e "${RED}❌ Build failed for $description${NC}"
        return 1
    fi
}

# Function to create Linux executable package
create_linux_package() {
    echo -e "${BLUE}📱 Creating Linux executable package...${NC}"

    local platform_dir="$RELEASE_DIR/ffmpegrust-linux-x86_64"
    mkdir -p "$platform_dir"

    # Copy binary
    cp "target/x86_64-unknown-linux-gnu/release/ffmpegrust" "$platform_dir/"
    chmod +x "$platform_dir/ffmpegrust"

    # Copy documentation
    cp README.md "$platform_dir/"
    cp LICENSE "$platform_dir/" 2>/dev/null || echo "LICENSE file not found"

    # Create run script
    cat > "$platform_dir/run.sh" << 'EOF'
#!/bin/bash
# FFmpeg Converter Pro Launcher

echo "Starting FFmpeg Converter Pro..."

# Check if FFmpeg is available
if ! command -v ffmpeg &> /dev/null; then
    echo ""
    echo "⚠️  FFmpeg not found!"
    echo ""
    echo "Please install FFmpeg using your package manager:"
    echo "  Ubuntu/Debian: sudo apt update && sudo apt install ffmpeg"
    echo "  Fedora/RHEL:   sudo dnf install ffmpeg"
    echo "  Arch Linux:    sudo pacman -S ffmpeg"
    echo "  openSUSE:      sudo zypper install ffmpeg"
    echo ""
    read -p "Press Enter to continue anyway or Ctrl+C to exit..."
fi

# Launch the application
./ffmpegrust
EOF
    chmod +x "$platform_dir/run.sh"

    # Create README for users
    cat > "$platform_dir/INSTALL.txt" << EOF
FFmpeg Converter Pro v${VERSION} - Linux Installation
====================================================

QUICK START:
1. Install FFmpeg (required):
   Ubuntu/Debian: sudo apt update && sudo apt install ffmpeg
   Fedora/RHEL:   sudo dnf install ffmpeg
   Arch Linux:    sudo pacman -S ffmpeg
   openSUSE:      sudo zypper install ffmpeg

2. Run the application:
   ./run.sh

   Or directly:
   ./ffmpegrust

FEATURES:
- Hardware acceleration support (NVIDIA NVENC, Intel QSV, AMD VCE)
- Real-time progress monitoring
- Fast remuxing without re-encoding
- Multiple quality presets
- Auto-update system

For support, visit: https://github.com/paterkleomenis/ffmpegrust
EOF

    # Create archive
    cd "$RELEASE_DIR"
    tar -czf "ffmpegrust-linux-x86_64.tar.gz" "ffmpegrust-linux-x86_64/"
    cd - > /dev/null

    echo -e "${GREEN}✅ Created Linux package: ffmpegrust-linux-x86_64.tar.gz${NC}"
}

# Function to create macOS executable package
create_macos_package() {
    echo -e "${BLUE}🍎 Creating macOS executable package...${NC}"

    local platform_dir="$RELEASE_DIR/ffmpegrust-macos-aarch64"
    mkdir -p "$platform_dir"

    # Copy binary
    cp "target/aarch64-apple-darwin/release/ffmpegrust" "$platform_dir/"
    chmod +x "$platform_dir/ffmpegrust"

    # Copy documentation
    cp README.md "$platform_dir/"
    cp LICENSE "$platform_dir/" 2>/dev/null || echo "LICENSE file not found"

    # Create run script
    cat > "$platform_dir/run.sh" << 'EOF'
#!/bin/bash
# FFmpeg Converter Pro Launcher

echo "Starting FFmpeg Converter Pro..."

# Check if FFmpeg is available
if ! command -v ffmpeg &> /dev/null; then
    echo ""
    echo "⚠️  FFmpeg not found!"
    echo ""
    echo "Installing FFmpeg via Homebrew..."

    # Check if Homebrew is installed
    if ! command -v brew &> /dev/null; then
        echo "Homebrew not found. Installing Homebrew first..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

        # Add Homebrew to PATH for this session
        if [[ -f "/opt/homebrew/bin/brew" ]]; then
            eval "$(/opt/homebrew/bin/brew shellenv)"
        elif [[ -f "/usr/local/bin/brew" ]]; then
            eval "$(/usr/local/bin/brew shellenv)"
        fi
    fi

    # Install FFmpeg
    if command -v brew &> /dev/null; then
        brew install ffmpeg
        echo "✅ FFmpeg installed successfully!"
    else
        echo "❌ Failed to install Homebrew. Please install FFmpeg manually:"
        echo "   Visit: https://ffmpeg.org/download.html"
        echo ""
        read -p "Press Enter to continue anyway or Ctrl+C to exit..."
    fi
fi

# Launch the application
./ffmpegrust
EOF
    chmod +x "$platform_dir/run.sh"

    # Create install script
    cat > "$platform_dir/install.sh" << 'EOF'
#!/bin/bash
# FFmpeg Converter Pro Installation Script

echo "Installing FFmpeg Converter Pro..."

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

    # Add Homebrew to PATH
    if [[ -f "/opt/homebrew/bin/brew" ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    elif [[ -f "/usr/local/bin/brew" ]]; then
        eval "$(/usr/local/bin/brew shellenv)"
    fi
fi

# Install FFmpeg
if command -v brew &> /dev/null; then
    echo "Installing FFmpeg..."
    brew install ffmpeg
    echo "✅ Installation complete!"
    echo ""
    echo "You can now run FFmpeg Converter Pro:"
    echo "  ./run.sh"
    echo "  or"
    echo "  ./ffmpegrust"
else
    echo "❌ Failed to install Homebrew"
    echo "Please install FFmpeg manually from: https://ffmpeg.org/download.html"
fi
EOF
    chmod +x "$platform_dir/install.sh"

    # Create README for users
    cat > "$platform_dir/INSTALL.txt" << EOF
FFmpeg Converter Pro v${VERSION} - macOS Installation
===================================================

QUICK START:
1. Run the install script (installs FFmpeg automatically):
   ./install.sh

2. Launch the application:
   ./run.sh

   Or directly:
   ./ffmpegrust

MANUAL INSTALLATION:
If automatic installation fails:
1. Install Homebrew: /bin/bash -c "\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
2. Install FFmpeg: brew install ffmpeg

SYSTEM REQUIREMENTS:
- macOS 11.0+ (Big Sur or later)
- Apple Silicon Mac (M1/M2/M3/M4)

FEATURES:
- Native Apple Silicon performance
- VideoToolbox hardware acceleration
- Real-time progress monitoring
- Fast remuxing without re-encoding
- Multiple quality presets
- Auto-update system

For support, visit: https://github.com/paterkleomenis/ffmpegrust
EOF

    # Create archive
    cd "$RELEASE_DIR"
    tar -czf "ffmpegrust-macos-aarch64.tar.gz" "ffmpegrust-macos-aarch64/"
    cd - > /dev/null

    echo -e "${GREEN}✅ Created macOS package: ffmpegrust-macos-aarch64.tar.gz${NC}"
}

# Function to create Windows installer
create_windows_installer() {
    echo -e "${BLUE}🪟 Creating Windows installer...${NC}"

    # Check if NSIS is available
    if ! command -v makensis &> /dev/null; then
        echo -e "${YELLOW}⚠️  NSIS not found. Creating basic Windows package instead...${NC}"
        create_windows_package
        return
    fi

    # Build the installer
    cd installer
    makensis windows-installer.nsi
    cd - > /dev/null

    # Move installer to release directory
    if [[ -f "installer/FFmpegConverterPro-Setup.exe" ]]; then
        mv "installer/FFmpegConverterPro-Setup.exe" "$RELEASE_DIR/"
        echo -e "${GREEN}✅ Created Windows installer: FFmpegConverterPro-Setup.exe${NC}"
    else
        echo -e "${RED}❌ Failed to create Windows installer${NC}"
        create_windows_package
    fi
}

# Function to create basic Windows package (fallback)
create_windows_package() {
    echo -e "${BLUE}📦 Creating Windows package...${NC}"

    local platform_dir="$RELEASE_DIR/ffmpegrust-windows-x86_64"
    mkdir -p "$platform_dir"

    # Copy binary
    cp "target/x86_64-pc-windows-msvc/release/ffmpegrust.exe" "$platform_dir/"

    # Copy documentation
    cp README.md "$platform_dir/"
    cp LICENSE "$platform_dir/" 2>/dev/null || echo "LICENSE file not found"

    # Create install script
    cat > "$platform_dir/install.bat" << 'EOF'
@echo off
echo Installing FFmpeg Converter Pro...
echo.

REM Check if FFmpeg is installed
where ffmpeg >nul 2>&1
if errorlevel 1 (
    echo FFmpeg not found. Installing FFmpeg...
    echo.

    REM Try winget first
    winget install --id=Gyan.FFmpeg -e --accept-source-agreements --accept-package-agreements
    if errorlevel 1 (
        echo Winget failed. Trying chocolatey...
        choco install ffmpeg -y
        if errorlevel 1 (
            echo.
            echo ❌ Automatic installation failed!
            echo.
            echo Please install FFmpeg manually:
            echo 1. Download from: https://www.gyan.dev/ffmpeg/builds/
            echo 2. Extract to C:\ffmpeg
            echo 3. Add C:\ffmpeg\bin to your PATH
            echo.
            pause
            exit /b 1
        )
    )
    echo.
    echo ✅ FFmpeg installed successfully!
) else (
    echo ✅ FFmpeg is already installed!
)

echo.
echo Installation complete!
echo You can now run ffmpegrust.exe or use run.bat
echo.
pause
EOF

    # Create run script
    cat > "$platform_dir/run.bat" << 'EOF'
@echo off
echo Starting FFmpeg Converter Pro...

REM Check if FFmpeg is available
where ffmpeg >nul 2>&1
if errorlevel 1 (
    echo.
    echo ⚠️  FFmpeg not found!
    echo Please run install.bat first to install FFmpeg
    echo.
    pause
    exit /b 1
)

REM Launch the application
ffmpegrust.exe
EOF

    # Create README for users
    cat > "$platform_dir/INSTALL.txt" << EOF
FFmpeg Converter Pro v${VERSION} - Windows Installation
======================================================

QUICK START:
1. Run install.bat (installs FFmpeg automatically)
2. Launch the application:
   - Double-click ffmpegrust.exe
   - Or run run.bat

MANUAL INSTALLATION:
If automatic installation fails:
1. Download FFmpeg from: https://www.gyan.dev/ffmpeg/builds/
2. Extract to C:\ffmpeg
3. Add C:\ffmpeg\bin to your system PATH

SYSTEM REQUIREMENTS:
- Windows 10/11 (64-bit)
- .NET Framework (usually pre-installed)

FEATURES:
- NVIDIA NVENC hardware acceleration
- Intel QSV hardware acceleration
- Real-time progress monitoring
- Fast remuxing without re-encoding
- Multiple quality presets
- Auto-update system

For support, visit: https://github.com/paterkleomenis/ffmpegrust
EOF

    # Create archive
    cd "$RELEASE_DIR"
    if command -v zip &> /dev/null; then
        zip -r "ffmpegrust-windows-x86_64.zip" "ffmpegrust-windows-x86_64/"
    else
        echo -e "${YELLOW}⚠️  zip not found, creating tar.gz instead${NC}"
        tar -czf "ffmpegrust-windows-x86_64.tar.gz" "ffmpegrust-windows-x86_64/"
    fi
    cd - > /dev/null

    echo -e "${GREEN}✅ Created Windows package${NC}"
}

# Main build process
echo -e "${BLUE}🔧 Building all platforms...${NC}"
echo ""

BUILD_SUCCESS=0
TOTAL_BUILDS=0

# Build Linux
echo -e "${BLUE}=== LINUX BUILD ===${NC}"
((TOTAL_BUILDS++))
if build_target "x86_64-unknown-linux-gnu" "linux-x86_64" "Linux x86_64"; then
    create_linux_package
    ((BUILD_SUCCESS++))
fi
echo ""

# Build macOS (only on macOS or if cross-compilation is set up)
echo -e "${BLUE}=== MACOS BUILD ===${NC}"
((TOTAL_BUILDS++))
if build_target "aarch64-apple-darwin" "macos-aarch64" "macOS ARM64 (Apple Silicon)"; then
    create_macos_package
    ((BUILD_SUCCESS++))
fi
echo ""

# Build Windows
echo -e "${BLUE}=== WINDOWS BUILD ===${NC}"
((TOTAL_BUILDS++))
if build_target "x86_64-pc-windows-msvc" "windows-x86_64" "Windows x86_64"; then
    create_windows_installer
    ((BUILD_SUCCESS++))
fi
echo ""

# Summary
echo -e "${BLUE}📊 Build Summary${NC}"
echo -e "Successful builds: ${GREEN}$BUILD_SUCCESS${NC}/${TOTAL_BUILDS}"
echo -e "Version: ${GREEN}v$VERSION${NC}"
echo ""

if [[ $BUILD_SUCCESS -gt 0 ]]; then
    echo -e "${GREEN}🎉 Release packages created in: $RELEASE_DIR${NC}"
    echo ""
    echo -e "${BLUE}📋 Release Files:${NC}"
    ls -la "$RELEASE_DIR"
    echo ""
    echo -e "${BLUE}📝 Upload Instructions:${NC}"
    echo -e "1. Go to: ${YELLOW}https://github.com/YOUR_USERNAME/ffmpegrust/releases${NC}"
    echo -e "2. Click: ${YELLOW}Create a new release${NC}"
    echo -e "3. Tag: ${YELLOW}v$VERSION${NC}"
    echo -e "4. Upload all files from: ${YELLOW}$RELEASE_DIR${NC}"
    echo -e "5. Publish the release"
    echo ""
    echo -e "${GREEN}✅ Ready for distribution!${NC}"
else
    echo -e "${RED}❌ No successful builds. Please check the errors above.${NC}"
    exit 1
fi
