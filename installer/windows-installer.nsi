; FFmpeg Converter Pro - Windows Installer
; NSIS Script for creating Windows installer

!include "MUI2.nsh"
!include "FileFunc.nsh"

; General settings
Name "FFmpeg Converter Pro"
OutFile "FFmpegConverterPro-Setup.exe"
InstallDir "$PROGRAMFILES64\FFmpeg Converter Pro"
InstallDirRegKey HKCU "Software\FFmpegConverterPro" ""
RequestExecutionLevel admin

; Version information
VIProductVersion "1.0.0.0"
VIAddVersionKey "ProductName" "FFmpeg Converter Pro"
VIAddVersionKey "CompanyName" "FFmpeg Converter Pro Team"
VIAddVersionKey "LegalCopyright" "© 2024 FFmpeg Converter Pro"
VIAddVersionKey "FileDescription" "FFmpeg Converter Pro Installer"
VIAddVersionKey "FileVersion" "1.0.0.0"
VIAddVersionKey "ProductVersion" "1.0.0"

; Modern UI configuration
!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

; Welcome page
!define MUI_WELCOMEPAGE_TITLE "Welcome to FFmpeg Converter Pro Setup"
!define MUI_WELCOMEPAGE_TEXT "This will install FFmpeg Converter Pro on your computer.$\r$\n$\r$\nFFmpeg Converter Pro is a modern, professional GUI application for video conversion and remuxing.$\r$\n$\r$\nClick Next to continue."
!insertmacro MUI_PAGE_WELCOME

; License page
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"

; Components page
!insertmacro MUI_PAGE_COMPONENTS

; Directory page
!insertmacro MUI_PAGE_DIRECTORY

; Installation page
!insertmacro MUI_PAGE_INSTFILES

; Finish page
!define MUI_FINISHPAGE_RUN "$INSTDIR\ffmpegrust.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch FFmpeg Converter Pro"
!define MUI_FINISHPAGE_LINK "Visit our website"
!define MUI_FINISHPAGE_LINK_LOCATION "https://github.com/paterkleomenis/ffmpegrust"
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Languages
!insertmacro MUI_LANGUAGE "English"

; Installer sections
Section "FFmpeg Converter Pro (Required)" SecMain
  SectionIn RO

  ; Set output path to the installation directory
  SetOutPath $INSTDIR

  ; Copy main executable
  File "..\target\release\ffmpegrust.exe"

  ; Copy documentation
  File "..\README.md"
  File "..\LICENSE"

  ; Store installation folder
  WriteRegStr HKCU "Software\FFmpegConverterPro" "" $INSTDIR

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Add to Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro" "DisplayName" "FFmpeg Converter Pro"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro" "UninstallString" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro" "Publisher" "FFmpeg Converter Pro Team"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro" "DisplayVersion" "1.0.0"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro" "DisplayIcon" "$INSTDIR\ffmpegrust.exe"
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro" "NoRepair" 1

  ; Get installation size
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro" "EstimatedSize" "$0"

SectionEnd

Section "Desktop Shortcut" SecDesktop
  CreateShortcut "$DESKTOP\FFmpeg Converter Pro.lnk" "$INSTDIR\ffmpegrust.exe"
SectionEnd

Section "Start Menu Shortcuts" SecStartMenu
  CreateDirectory "$SMPROGRAMS\FFmpeg Converter Pro"
  CreateShortcut "$SMPROGRAMS\FFmpeg Converter Pro\FFmpeg Converter Pro.lnk" "$INSTDIR\ffmpegrust.exe"
  CreateShortcut "$SMPROGRAMS\FFmpeg Converter Pro\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Add to PATH" SecPath
  ; Add installation directory to system PATH
  EnVar::SetHKLM
  EnVar::AddValue "PATH" "$INSTDIR"
  Pop $0
  DetailPrint "Added to PATH: $0"
SectionEnd

Section "Install FFmpeg" SecFFmpeg
  DetailPrint "Checking for FFmpeg..."

  ; Check if FFmpeg is already installed
  nsExec::ExecToStack 'where ffmpeg'
  Pop $0
  ${If} $0 == 0
    DetailPrint "FFmpeg is already installed"
  ${Else}
    DetailPrint "FFmpeg not found. Installing via winget..."

    ; Try to install FFmpeg using winget
    nsExec::ExecToLog 'winget install --id=Gyan.FFmpeg -e --accept-source-agreements --accept-package-agreements'
    Pop $0

    ${If} $0 == 0
      DetailPrint "FFmpeg installed successfully via winget"
    ${Else}
      DetailPrint "Winget installation failed. Trying chocolatey..."

      ; Try chocolatey as fallback
      nsExec::ExecToLog 'choco install ffmpeg -y'
      Pop $0

      ${If} $0 == 0
        DetailPrint "FFmpeg installed successfully via chocolatey"
      ${Else}
        DetailPrint "Automatic FFmpeg installation failed"
        MessageBox MB_OK|MB_ICONINFORMATION "FFmpeg installation failed.$\r$\n$\r$\nPlease install FFmpeg manually:$\r$\n1. Download from: https://www.gyan.dev/ffmpeg/builds/$\r$\n2. Extract to C:\ffmpeg$\r$\n3. Add C:\ffmpeg\bin to your PATH"
      ${EndIf}
    ${EndIf}
  ${EndIf}
SectionEnd

; Component descriptions
LangString DESC_SecMain ${LANG_ENGLISH} "The main FFmpeg Converter Pro application"
LangString DESC_SecDesktop ${LANG_ENGLISH} "Create a desktop shortcut"
LangString DESC_SecStartMenu ${LANG_ENGLISH} "Create Start Menu shortcuts"
LangString DESC_SecPath ${LANG_ENGLISH} "Add FFmpeg Converter Pro to system PATH"
LangString DESC_SecFFmpeg ${LANG_ENGLISH} "Automatically install FFmpeg (required for video conversion)"

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} $(DESC_SecMain)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_SecDesktop)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} $(DESC_SecStartMenu)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecPath} $(DESC_SecPath)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecFFmpeg} $(DESC_SecFFmpeg)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; Uninstaller section
Section "Uninstall"
  ; Remove files
  Delete "$INSTDIR\ffmpegrust.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\Uninstall.exe"

  ; Remove shortcuts
  Delete "$DESKTOP\FFmpeg Converter Pro.lnk"
  Delete "$SMPROGRAMS\FFmpeg Converter Pro\FFmpeg Converter Pro.lnk"
  Delete "$SMPROGRAMS\FFmpeg Converter Pro\Uninstall.lnk"
  RMDir "$SMPROGRAMS\FFmpeg Converter Pro"

  ; Remove from PATH
  EnVar::SetHKLM
  EnVar::DeleteValue "PATH" "$INSTDIR"
  Pop $0

  ; Remove registry keys
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FFmpegConverterPro"
  DeleteRegKey HKCU "Software\FFmpegConverterPro"

  ; Remove installation directory
  RMDir "$INSTDIR"

SectionEnd

; Installer initialization
Function .onInit
  ; Check Windows version
  ${IfNot} ${AtLeastWin10}
    MessageBox MB_OK|MB_ICONSTOP "FFmpeg Converter Pro requires Windows 10 or later."
    Abort
  ${EndIf}

  ; Check if already installed
  ReadRegStr $R0 HKCU "Software\FFmpegConverterPro" ""
  ${If} $R0 != ""
    MessageBox MB_YESNO|MB_ICONQUESTION "FFmpeg Converter Pro is already installed. Do you want to reinstall?" IDYES continue
    Abort
    continue:
  ${EndIf}
FunctionEnd
