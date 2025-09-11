@echo off
setlocal enabledelayedexpansion

REM FFmpeg Converter Pro - Windows Installer Builder
REM This script builds the Windows installer package

echo.
echo 🚀 FFmpeg Converter Pro - Windows Installer Builder
echo.

REM Get version from Cargo.toml
for /f "tokens=3 delims= " %%i in ('findstr "^version = " Cargo.toml') do (
    set VERSION=%%i
    set VERSION=!VERSION:"=!
)

echo Building Windows Installer v!VERSION!
echo.

REM Create release directory
set RELEASE_DIR=releases\v!VERSION!
if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"

echo 📦 Building Windows binary...
cargo build --release --target x86_64-pc-windows-msvc

if not exist "target\x86_64-pc-windows-msvc\release\ffmpegrust.exe" (
    echo ❌ Build failed! No binary found.
    pause
    exit /b 1
)

echo ✅ Build successful!
echo.

REM Check for NSIS
where makensis >nul 2>&1
if not errorlevel 1 (
    echo 🛠️  NSIS found. Building installer...

    REM Build NSIS installer
    cd installer
    makensis windows-installer.nsi
    cd ..

    if exist "installer\FFmpegConverterPro-Setup.exe" (
        move "installer\FFmpegConverterPro-Setup.exe" "%RELEASE_DIR%\"
        echo ✅ Windows installer created: FFmpegConverterPro-Setup.exe
        goto :summary
    ) else (
        echo ❌ NSIS build failed. Creating package instead...
        goto :create_package
    )
) else (
    echo ⚠️  NSIS not found. Creating package instead...
    goto :create_package
)

:create_package
echo 📦 Creating Windows package...

REM Create platform-specific directory
set PLATFORM_DIR=%RELEASE_DIR%\ffmpegrust-windows-x86_64
mkdir "%PLATFORM_DIR%"

echo 📋 Copying files...

REM Copy binary
copy "target\x86_64-pc-windows-msvc\release\ffmpegrust.exe" "%PLATFORM_DIR%\"
echo   ✅ ffmpegrust.exe

REM Copy documentation
copy "README.md" "%PLATFORM_DIR%\"
echo   ✅ README.md

if exist "LICENSE" (
    copy "LICENSE" "%PLATFORM_DIR%\"
    echo   ✅ LICENSE
)

REM Create install script
echo @echo off > "%PLATFORM_DIR%\install.bat"
echo echo Installing FFmpeg Converter Pro... >> "%PLATFORM_DIR%\install.bat"
echo echo. >> "%PLATFORM_DIR%\install.bat"
echo. >> "%PLATFORM_DIR%\install.bat"
echo REM Check if FFmpeg is installed >> "%PLATFORM_DIR%\install.bat"
echo where ffmpeg ^>nul 2^>^&1 >> "%PLATFORM_DIR%\install.bat"
echo if errorlevel 1 ^( >> "%PLATFORM_DIR%\install.bat"
echo     echo FFmpeg not found. Installing FFmpeg... >> "%PLATFORM_DIR%\install.bat"
echo     echo. >> "%PLATFORM_DIR%\install.bat"
echo. >> "%PLATFORM_DIR%\install.bat"
echo     REM Try winget first >> "%PLATFORM_DIR%\install.bat"
echo     winget install --id=Gyan.FFmpeg -e --accept-source-agreements --accept-package-agreements >> "%PLATFORM_DIR%\install.bat"
echo     if errorlevel 1 ^( >> "%PLATFORM_DIR%\install.bat"
echo         echo Winget failed. Trying chocolatey... >> "%PLATFORM_DIR%\install.bat"
echo         choco install ffmpeg -y >> "%PLATFORM_DIR%\install.bat"
echo         if errorlevel 1 ^( >> "%PLATFORM_DIR%\install.bat"
echo             echo. >> "%PLATFORM_DIR%\install.bat"
echo             echo ❌ Automatic installation failed! >> "%PLATFORM_DIR%\install.bat"
echo             echo. >> "%PLATFORM_DIR%\install.bat"
echo             echo Please install FFmpeg manually: >> "%PLATFORM_DIR%\install.bat"
echo             echo 1. Download from: https://www.gyan.dev/ffmpeg/builds/ >> "%PLATFORM_DIR%\install.bat"
echo             echo 2. Extract to C:\ffmpeg >> "%PLATFORM_DIR%\install.bat"
echo             echo 3. Add C:\ffmpeg\bin to your PATH >> "%PLATFORM_DIR%\install.bat"
echo             echo. >> "%PLATFORM_DIR%\install.bat"
echo             pause >> "%PLATFORM_DIR%\install.bat"
echo             exit /b 1 >> "%PLATFORM_DIR%\install.bat"
echo         ^) >> "%PLATFORM_DIR%\install.bat"
echo     ^) >> "%PLATFORM_DIR%\install.bat"
echo     echo. >> "%PLATFORM_DIR%\install.bat"
echo     echo ✅ FFmpeg installed successfully! >> "%PLATFORM_DIR%\install.bat"
echo ^) else ^( >> "%PLATFORM_DIR%\install.bat"
echo     echo ✅ FFmpeg is already installed! >> "%PLATFORM_DIR%\install.bat"
echo ^) >> "%PLATFORM_DIR%\install.bat"
echo. >> "%PLATFORM_DIR%\install.bat"
echo echo. >> "%PLATFORM_DIR%\install.bat"
echo echo Installation complete! >> "%PLATFORM_DIR%\install.bat"
echo echo You can now run ffmpegrust.exe or use run.bat >> "%PLATFORM_DIR%\install.bat"
echo echo. >> "%PLATFORM_DIR%\install.bat"
echo pause >> "%PLATFORM_DIR%\install.bat"

REM Create run script
echo @echo off > "%PLATFORM_DIR%\run.bat"
echo echo Starting FFmpeg Converter Pro... >> "%PLATFORM_DIR%\run.bat"
echo. >> "%PLATFORM_DIR%\run.bat"
echo REM Check if FFmpeg is available >> "%PLATFORM_DIR%\run.bat"
echo where ffmpeg ^>nul 2^>^&1 >> "%PLATFORM_DIR%\run.bat"
echo if errorlevel 1 ^( >> "%PLATFORM_DIR%\run.bat"
echo     echo. >> "%PLATFORM_DIR%\run.bat"
echo     echo ⚠️  FFmpeg not found! >> "%PLATFORM_DIR%\run.bat"
echo     echo Please run install.bat first to install FFmpeg >> "%PLATFORM_DIR%\run.bat"
echo     echo. >> "%PLATFORM_DIR%\run.bat"
echo     pause >> "%PLATFORM_DIR%\run.bat"
echo     exit /b 1 >> "%PLATFORM_DIR%\run.bat"
echo ^) >> "%PLATFORM_DIR%\run.bat"
echo. >> "%PLATFORM_DIR%\run.bat"
echo REM Launch the application >> "%PLATFORM_DIR%\run.bat"
echo ffmpegrust.exe >> "%PLATFORM_DIR%\run.bat"

REM Create user instructions
echo FFmpeg Converter Pro v!VERSION! - Windows Installation > "%PLATFORM_DIR%\INSTALL.txt"
echo ====================================================== >> "%PLATFORM_DIR%\INSTALL.txt"
echo. >> "%PLATFORM_DIR%\INSTALL.txt"
echo QUICK START: >> "%PLATFORM_DIR%\INSTALL.txt"
echo 1. Run install.bat (installs FFmpeg automatically) >> "%PLATFORM_DIR%\INSTALL.txt"
echo 2. Launch the application: >> "%PLATFORM_DIR%\INSTALL.txt"
echo    - Double-click ffmpegrust.exe >> "%PLATFORM_DIR%\INSTALL.txt"
echo    - Or run run.bat >> "%PLATFORM_DIR%\INSTALL.txt"
echo. >> "%PLATFORM_DIR%\INSTALL.txt"
echo MANUAL INSTALLATION: >> "%PLATFORM_DIR%\INSTALL.txt"
echo If automatic installation fails: >> "%PLATFORM_DIR%\INSTALL.txt"
echo 1. Download FFmpeg from: https://www.gyan.dev/ffmpeg/builds/ >> "%PLATFORM_DIR%\INSTALL.txt"
echo 2. Extract to C:\ffmpeg >> "%PLATFORM_DIR%\INSTALL.txt"
echo 3. Add C:\ffmpeg\bin to your system PATH >> "%PLATFORM_DIR%\INSTALL.txt"
echo. >> "%PLATFORM_DIR%\INSTALL.txt"
echo SYSTEM REQUIREMENTS: >> "%PLATFORM_DIR%\INSTALL.txt"
echo - Windows 10/11 (64-bit) >> "%PLATFORM_DIR%\INSTALL.txt"
echo - .NET Framework (usually pre-installed) >> "%PLATFORM_DIR%\INSTALL.txt"
echo. >> "%PLATFORM_DIR%\INSTALL.txt"
echo FEATURES: >> "%PLATFORM_DIR%\INSTALL.txt"
echo - NVIDIA NVENC hardware acceleration >> "%PLATFORM_DIR%\INSTALL.txt"
echo - Intel QSV hardware acceleration >> "%PLATFORM_DIR%\INSTALL.txt"
echo - Real-time progress monitoring >> "%PLATFORM_DIR%\INSTALL.txt"
echo - Fast remuxing without re-encoding >> "%PLATFORM_DIR%\INSTALL.txt"
echo - Multiple quality presets >> "%PLATFORM_DIR%\INSTALL.txt"
echo - Auto-update system >> "%PLATFORM_DIR%\INSTALL.txt"
echo. >> "%PLATFORM_DIR%\INSTALL.txt"
echo For support, visit: https://github.com/paterkleomenis/ffmpegrust >> "%PLATFORM_DIR%\INSTALL.txt"

echo   ✅ install.bat
echo   ✅ run.bat
echo   ✅ INSTALL.txt

echo.
echo 📦 Creating archive...

cd "%RELEASE_DIR%"
powershell -Command "Compress-Archive -Path 'ffmpegrust-windows-x86_64' -DestinationPath 'ffmpegrust-windows-x86_64.zip' -Force"
set ARCHIVE_FILE=ffmpegrust-windows-x86_64.zip
cd ..

echo ✅ Created Windows package: %ARCHIVE_FILE%

:summary
echo.
echo 📊 Windows Release Summary
echo   Platform: Windows x86_64
echo   Version: v!VERSION!

if exist "%RELEASE_DIR%\FFmpegConverterPro-Setup.exe" (
    echo   Type: Professional Installer
    echo   File: FFmpegConverterPro-Setup.exe
) else (
    echo   Type: Portable Package
    echo   File: %ARCHIVE_FILE%
)

echo.
echo 🎉 Windows release created successfully!
echo.
echo 📋 Files in %RELEASE_DIR%:
dir "%RELEASE_DIR%" /b
echo.
echo 📝 Upload Instructions:
echo 1. Go to: https://github.com/YOUR_USERNAME/ffmpegrust/releases
echo 2. Click: Create a new release
echo 3. Tag: v!VERSION!
echo 4. Upload all files from: %RELEASE_DIR%
echo 5. Publish the release
echo.
echo ✅ Ready for distribution!

pause
