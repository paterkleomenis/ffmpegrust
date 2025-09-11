#!/bin/bash

# FFmpeg Converter Pro - Local Release Builder
# This script creates release packages for all supported platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 FFmpeg Converter Pro - Release Builder${NC}"
echo ""

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | cut -d '"' -f 2)
echo -e "${BLUE}Building version: ${GREEN}v${VERSION}${NC}"
echo ""

# Create release directory
RELEASE_DIR="releases/v${VERSION}"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

echo -e "${BLUE}📦 Building release binary...${NC}"
cargo build --release

if [ ! -f "target/release/ffmpegrust" ] && [ ! -f "target/release/ffmpegrust.exe" ]; then
    echo -e "${RED}❌ Build failed! No binary found.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build successful!${NC}"
echo ""

# Detect current platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux-x86_64"
    BINARY="ffmpegrust"
    ARCHIVE_CMD="tar -czf"
    ARCHIVE_EXT="tar.gz"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macos-aarch64"
    BINARY="ffmpegrust"
    ARCHIVE_CMD="tar -czf"
    ARCHIVE_EXT="tar.gz"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    PLATFORM="windows-x86_64"
    BINARY="ffmpegrust.exe"
    ARCHIVE_CMD="zip -r"
    ARCHIVE_EXT="zip"
else
    echo -e "${RED}❌ Unsupported platform: $OSTYPE${NC}"
    exit 1
fi

echo -e "${BLUE}📱 Detected platform: ${GREEN}$PLATFORM${NC}"
echo ""

# Create platform-specific directory
PLATFORM_DIR="$RELEASE_DIR/ffmpegrust-$PLATFORM"
mkdir -p "$PLATFORM_DIR"

echo -e "${BLUE}📋 Copying files...${NC}"

# Copy binary
cp "target/release/$BINARY" "$PLATFORM_DIR/"
echo -e "  ✅ Binary: $BINARY"

# Copy additional files
cp README.md "$PLATFORM_DIR/"
echo -e "  ✅ README.md"

if [ -f "LICENSE" ]; then
    cp LICENSE "$PLATFORM_DIR/"
    echo -e "  ✅ LICENSE"
fi

# Create installation script based on platform
if [[ "$PLATFORM" == "windows-x86_64" ]]; then
    cat > "$PLATFORM_DIR/install.bat" << 'EOF'
@echo off
echo Installing FFmpeg Converter Pro...

REM Check if FFmpeg is installed
where ffmpeg >nul 2>&1
if errorlevel 1 (
    echo FFmpeg not found. Installing FFmpeg...
    winget install --id=Gyan.FFmpeg -e
    if errorlevel 1 (
        echo Failed to install FFmpeg automatically.
        echo Please install FFmpeg manually from: https://www.gyan.dev/ffmpeg/builds/
        pause
        exit /b 1
    )
) else (
    echo FFmpeg is already installed.
)

echo Installation complete!
echo You can now run ffmpegrust.exe
pause
EOF

    cat > "$PLATFORM_DIR/run.bat" << 'EOF'
@echo off
REM Check if FFmpeg is available
where ffmpeg >nul 2>&1
if errorlevel 1 (
    echo FFmpeg not found! Please run install.bat first.
    pause
    exit /b 1
)
ffmpegrust.exe
EOF
    echo -e "  ✅ install.bat"
    echo -e "  ✅ run.bat"

elif [[ "$PLATFORM" == "macos-aarch64" ]]; then
    cat > "$PLATFORM_DIR/install.sh" << 'EOF'
#!/bin/bash
echo "Installing FFmpeg Converter Pro..."

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "Homebrew not found. Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Check if FFmpeg is installed
if ! command -v ffmpeg &> /dev/null; then
    echo "FFmpeg not found. Installing FFmpeg..."
    brew install ffmpeg
else
    echo "FFmpeg is already installed."
fi

echo "Installation complete!"
echo "You can now run ./ffmpegrust"
EOF
    chmod +x "$PLATFORM_DIR/install.sh"

    cat > "$PLATFORM_DIR/run.sh" << 'EOF'
#!/bin/bash
# Check if FFmpeg is available
if ! command -v ffmpeg &> /dev/null; then
    echo "FFmpeg not found! Please run ./install.sh first."
    exit 1
fi
./ffmpegrust
EOF
    chmod +x "$PLATFORM_DIR/run.sh"
    chmod +x "$PLATFORM_DIR/ffmpegrust"
    echo -e "  ✅ install.sh"
    echo -e "  ✅ run.sh"

else # Linux
    cat > "$PLATFORM_DIR/run.sh" << 'EOF'
#!/bin/bash
# Check if FFmpeg is available
if ! command -v ffmpeg &> /dev/null; then
    echo "FFmpeg not found! Please install it using your package manager:"
    echo "  Ubuntu/Debian: sudo apt update && sudo apt install ffmpeg"
    echo "  Fedora/RHEL: sudo dnf install ffmpeg"
    echo "  Arch Linux: sudo pacman -S ffmpeg"
    exit 1
fi
./ffmpegrust
EOF
    chmod +x "$PLATFORM_DIR/run.sh"
    chmod +x "$PLATFORM_DIR/ffmpegrust"
    echo -e "  ✅ run.sh"
fi

echo ""
echo -e "${BLUE}📦 Creating archive...${NC}"

cd "$RELEASE_DIR"

if [[ "$PLATFORM" == "windows-x86_64" ]]; then
    zip -r "ffmpegrust-$PLATFORM.zip" "ffmpegrust-$PLATFORM/"
    ARCHIVE_FILE="ffmpegrust-$PLATFORM.zip"
else
    tar -czf "ffmpegrust-$PLATFORM.tar.gz" "ffmpegrust-$PLATFORM/"
    ARCHIVE_FILE="ffmpegrust-$PLATFORM.tar.gz"
fi

cd - > /dev/null

echo -e "${GREEN}✅ Created: $RELEASE_DIR/$ARCHIVE_FILE${NC}"
echo ""

# Calculate file size
ARCHIVE_PATH="$RELEASE_DIR/$ARCHIVE_FILE"
if [[ "$OSTYPE" == "darwin"* ]]; then
    SIZE=$(stat -f%z "$ARCHIVE_PATH")
else
    SIZE=$(stat -c%s "$ARCHIVE_PATH")
fi
SIZE_MB=$((SIZE / 1024 / 1024))

echo -e "${BLUE}📊 Release Summary${NC}"
echo -e "  Platform: $PLATFORM"
echo -e "  Version: v$VERSION"
echo -e "  Archive: $ARCHIVE_FILE"
echo -e "  Size: ${SIZE_MB}MB"
echo ""

echo -e "${GREEN}🎉 Release package created successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Next steps:${NC}"
echo -e "1. Test the release: extract and run $ARCHIVE_FILE"
echo -e "2. Upload to GitHub:"
echo -e "   - Go to: https://github.com/YOUR_USERNAME/ffmpegrust/releases"
echo -e "   - Click 'Create a new release'"
echo -e "   - Tag: v$VERSION"
echo -e "   - Upload: $RELEASE_DIR/$ARCHIVE_FILE"
echo -e "3. Or create git tag for automatic release:"
echo -e "   git tag v$VERSION && git push origin v$VERSION"
echo ""
