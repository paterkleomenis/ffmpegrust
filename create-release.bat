@echo off
setlocal enabledelayedexpansion

REM FFmpeg Converter Pro - Local Release Builder (Windows)
REM This script creates release packages for Windows

echo.
echo 🚀 FFmpeg Converter Pro - Release Builder
echo.

REM Get version from Cargo.toml
for /f "tokens=3 delims= " %%i in ('findstr "^version = " Cargo.toml') do (
    set VERSION=%%i
    set VERSION=!VERSION:"=!
)

echo Building version: v!VERSION!
echo.

REM Create release directory
set RELEASE_DIR=releases\v!VERSION!
if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"

echo 📦 Building release binary...
cargo build --release

if not exist "target\release\ffmpegrust.exe" (
    echo ❌ Build failed! No binary found.
    pause
    exit /b 1
)

echo ✅ Build successful!
echo.

REM Set platform info
set PLATFORM=windows-x86_64
set BINARY=ffmpegrust.exe

echo 📱 Platform: %PLATFORM%
echo.

REM Create platform-specific directory
set PLATFORM_DIR=%RELEASE_DIR%\ffmpegrust-%PLATFORM%
mkdir "%PLATFORM_DIR%"

echo 📋 Copying files...

REM Copy binary
copy "target\release\%BINARY%" "%PLATFORM_DIR%\"
echo   ✅ Binary: %BINARY%

REM Copy additional files
copy "README.md" "%PLATFORM_DIR%\"
echo   ✅ README.md

if exist "LICENSE" (
    copy "LICENSE" "%PLATFORM_DIR%\"
    echo   ✅ LICENSE
)

REM Create installation script
echo @echo off > "%PLATFORM_DIR%\install.bat"
echo echo Installing FFmpeg Converter Pro... >> "%PLATFORM_DIR%\install.bat"
echo. >> "%PLATFORM_DIR%\install.bat"
echo REM Check if FFmpeg is installed >> "%PLATFORM_DIR%\install.bat"
echo where ffmpeg ^>nul 2^>^&1 >> "%PLATFORM_DIR%\install.bat"
echo if errorlevel 1 ^( >> "%PLATFORM_DIR%\install.bat"
echo     echo FFmpeg not found. Installing FFmpeg... >> "%PLATFORM_DIR%\install.bat"
echo     winget install --id=Gyan.FFmpeg -e >> "%PLATFORM_DIR%\install.bat"
echo     if errorlevel 1 ^( >> "%PLATFORM_DIR%\install.bat"
echo         echo Failed to install FFmpeg automatically. >> "%PLATFORM_DIR%\install.bat"
echo         echo Please install FFmpeg manually from: https://www.gyan.dev/ffmpeg/builds/ >> "%PLATFORM_DIR%\install.bat"
echo         pause >> "%PLATFORM_DIR%\install.bat"
echo         exit /b 1 >> "%PLATFORM_DIR%\install.bat"
echo     ^) >> "%PLATFORM_DIR%\install.bat"
echo ^) else ^( >> "%PLATFORM_DIR%\install.bat"
echo     echo FFmpeg is already installed. >> "%PLATFORM_DIR%\install.bat"
echo ^) >> "%PLATFORM_DIR%\install.bat"
echo. >> "%PLATFORM_DIR%\install.bat"
echo echo Installation complete! >> "%PLATFORM_DIR%\install.bat"
echo echo You can now run ffmpegrust.exe >> "%PLATFORM_DIR%\install.bat"
echo pause >> "%PLATFORM_DIR%\install.bat"

REM Create run script
echo @echo off > "%PLATFORM_DIR%\run.bat"
echo REM Check if FFmpeg is available >> "%PLATFORM_DIR%\run.bat"
echo where ffmpeg ^>nul 2^>^&1 >> "%PLATFORM_DIR%\run.bat"
echo if errorlevel 1 ^( >> "%PLATFORM_DIR%\run.bat"
echo     echo FFmpeg not found! Please run install.bat first. >> "%PLATFORM_DIR%\run.bat"
echo     pause >> "%PLATFORM_DIR%\run.bat"
echo     exit /b 1 >> "%PLATFORM_DIR%\run.bat"
echo ^) >> "%PLATFORM_DIR%\run.bat"
echo ffmpegrust.exe >> "%PLATFORM_DIR%\run.bat"

echo   ✅ install.bat
echo   ✅ run.bat

echo.
echo 📦 Creating archive...

cd "%RELEASE_DIR%"
powershell -Command "Compress-Archive -Path 'ffmpegrust-%PLATFORM%' -DestinationPath 'ffmpegrust-%PLATFORM%.zip' -Force"
set ARCHIVE_FILE=ffmpegrust-%PLATFORM%.zip
cd ..

echo ✅ Created: %RELEASE_DIR%\%ARCHIVE_FILE%
echo.

REM Get file size
for %%A in ("%RELEASE_DIR%\%ARCHIVE_FILE%") do set SIZE=%%~zA
set /a SIZE_MB=!SIZE! / 1024 / 1024

echo 📊 Release Summary
echo   Platform: %PLATFORM%
echo   Version: v!VERSION!
echo   Archive: %ARCHIVE_FILE%
echo   Size: !SIZE_MB!MB
echo.

echo 🎉 Release package created successfully!
echo.
echo 📋 Next steps:
echo 1. Test the release: extract and run %ARCHIVE_FILE%
echo 2. Upload to GitHub:
echo    - Go to: https://github.com/YOUR_USERNAME/ffmpegrust/releases
echo    - Click 'Create a new release'
echo    - Tag: v!VERSION!
echo    - Upload: %RELEASE_DIR%\%ARCHIVE_FILE%
echo 3. Or create git tag for automatic release:
echo    git tag v!VERSION! ^&^& git push origin v!VERSION!
echo.

pause
