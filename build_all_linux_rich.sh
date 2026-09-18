#!/bin/bash
set -e

APP_NAME="caterm"
APP_ID="com.cecepazhar.caterm"
VERSION="1.2.0"
DIST_DIR="dist"

mkdir -p "$DIST_DIR"

echo "Building Linux packages with Rich AppStream Metadata..."
podman run --rm -i -v "$PWD":/app:z -w /app debian:12-slim bash -c "
  apt-get update && apt-get install -y curl tar dpkg-dev rpm ruby ruby-dev build-essential
  gem install --no-document fpm

  TMP_PKG=\"/tmp/caterm_pkg\"
  rm -rf \"\$TMP_PKG\"
  
  # Directory structure
  mkdir -p \"\$TMP_PKG/usr/bin\"
  mkdir -p \"\$TMP_PKG/usr/share/applications\"
  mkdir -p \"\$TMP_PKG/usr/share/icons/hicolor/512x512/apps\"
  mkdir -p \"\$TMP_PKG/usr/share/metainfo\"
  
  # Binary
  cp build/bin/caterm \"\$TMP_PKG/usr/bin/\"
  chmod +x \"\$TMP_PKG/usr/bin/caterm\"

  # Icon
  cp build/appicon.png \"\$TMP_PKG/usr/share/icons/hicolor/512x512/apps/\.png\"

  # Desktop file
  cat << 'DESKTOP' > \"\$TMP_PKG/usr/share/applications/\.desktop\"
[Desktop Entry]
Name=CATerm
Comment=Zero-Knowledge SSH Connection Manager
Exec=/usr/bin/caterm
Icon=\
Terminal=false
Type=Application
Categories=Utility;System;Network;
DESKTOP

  # AppStream Metadata (Required for beautiful Software Center pages)
  cat << 'APPSTREAM' > \"\$TMP_PKG/usr/share/metainfo/\.metainfo.xml\"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<component type=\"desktop-application\">
  <id>\</id>
  <name>CATerm</name>
  <summary>Zero-Knowledge SSH Connection Manager</summary>
  <description>
    <p>CATerm is a modern, cross-platform SSH connection manager built for developers and sysadmins who prioritize security and productivity.</p>
    <p>Key features include:</p>
    <ul>
      <li>Zero-Knowledge Encryption: All SSH keys and passwords are encrypted locally. The master password is never transmitted.</li>
      <li>Trust On First Use (TOFU): Prevents MITM attacks by persistently pinning SSH host keys.</li>
      <li>Integrated PTY Terminal: Full-color xterm.js terminal integration right in the app.</li>
      <li>AI Copilot: Ask AI to explain commands, generate scripts, or troubleshoot logs directly from the terminal workspace.</li>
    </ul>
  </description>
  <url type=\"homepage\">https://github.com/cecep-azhar/caterm</url>
  <url type=\"bugtracker\">https://github.com/cecep-azhar/caterm/issues</url>
  <developer id=\"com.cecepazhar\">
    <name>Cecep Saeful Azhar</name>
  </developer>
  <launchable type=\"desktop-id\">\.desktop</launchable>
  <categories>
    <category>Network</category>
    <category>System</category>
    <category>Utility</category>
  </categories>
  <provides>
    <binary>caterm</binary>
  </provides>
  <project_license>MIT</project_license>
  <metadata_license>CC0-1.0</metadata_license>
  <content_rating type=\"oars-1.1\"/>
  <releases>
    <release version=\"$VERSION\" date=\"$(date +%Y-%m-%d)\">
      <description>
        <p>Security Phase 1 update: TOFU SSH Key Verification, AI Opt-In, and strict SQLite Data Encryption.</p>
      </description>
    </release>
  </releases>
</component>
APPSTREAM

  # Build DEB
  fpm -s dir -t deb -n caterm -v $VERSION \\
      --vendor \"Cecep Saeful Azhar\" \\
      --maintainer \"hi@cecepazhar.com\" \\
      --license \"MIT\" \\
      --url \"https://github.com/cecep-azhar/caterm\" \\
      --description \"Zero-Knowledge SSH Connection Manager with AI Copilot\" \\
      -C \"\$TMP_PKG\" --prefix / -p dist/caterm__amd64.deb

  # Build RPM
  fpm -s dir -t rpm -n caterm -v $VERSION \\
      --vendor \"Cecep Saeful Azhar\" \\
      --maintainer \"hi@cecepazhar.com\" \\
      --license \"MIT\" \\
      --url \"https://github.com/cecep-azhar/caterm\" \\
      --description \"Zero-Knowledge SSH Connection Manager with AI Copilot\" \\
      -d \"webkit2gtk4.1\" \\
      -C \"\$TMP_PKG\" --prefix / -p dist/caterm--1.x86_64.rpm
"

echo "Metadata injection complete!"
