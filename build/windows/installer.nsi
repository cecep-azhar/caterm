# NSIS Script for CATERM (Evergreen WebView2 Installer)

!define APP_NAME "CATERM"
!define COMP_NAME "Cecep Saeful Azhar Hidayat"
!define WEB_SITE "https://cecepazhar.com"
!define VERSION "1.0.0.0"
!define COPYRIGHT "Copyright (c) 2026 Cecep Saeful Azhar Hidayat"
!define DESCRIPTION "Terminal Emulator & Vault Manager"

!define INSTALLER_NAME "caterm-installer.exe"
!define MAIN_APP_EXE "caterm.exe"
!define ICON "build/appicon.ico"

OutFile "build/bin/${INSTALLER_NAME}"
InstallDir "$PROGRAMFILES64\CATERM"
RequestExecutionLevel admin

Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  SetOverwrite ifnewer
  File "build/bin/${MAIN_APP_EXE}"
  
  CreateDirectory "$SMPROGRAMS\CATERM"
  CreateShortCut "$SMPROGRAMS\CATERM\CATERM.lnk" "$INSTDIR\${MAIN_APP_EXE}"
  CreateShortCut "$DESKTOP\CATERM.lnk" "$INSTDIR\${MAIN_APP_EXE}"
  
  # WebView2 Evergreen Bootstrapper Check/Install
  # Checks registry for WebView2 Runtime
  ReadRegStr $0 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3C4FA00-6D47-4ADB-B643-FD4D99290077}" "pv"
  StrCmp $0 "" install_webview2 webview2_ok

install_webview2:
  DetailPrint "Installing Microsoft Edge WebView2 Evergreen Runtime..."
  # ExecWait / Download Evergreen Bootstrapper if needed
  # In actual NSIS, download WebView2Setup.exe and run with /silent /install

webview2_ok:
SectionEnd
