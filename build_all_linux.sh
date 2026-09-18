#!/bin/bash
set -e

APP_NAME="caterm"
VERSION="1.2.0"
DIST_DIR="dist"

mkdir -p "$DIST_DIR"

echo "Building Linux amd64 binary with webkit2_41 (Fedora 39+/Debian 12+ compatible)..."
podman run --rm -i -v "$PWD":/app:z -w /app debian:12-slim bash -c "
  apt-get update && apt-get install -y \
    curl gcc g++ make git pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev \
    tar wget dpkg-dev rpm ruby ruby-dev build-essential
  
  gem install --no-document fpm

  curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
  apt-get install -y nodejs
  
  curl -fsSL https://go.dev/dl/go1.22.4.linux-amd64.tar.gz | tar -C /usr/local -xz
  export PATH=/usr/local/go/bin:\$PATH
  
  go install github.com/wailsapp/wails/v2/cmd/wails@latest
  export PATH=\$(go env GOPATH)/bin:\$PATH
  
  # Build wails binary with webkit2_41
  wails build -platform linux/amd64 -tags webkit2_41 -clean -ldflags '-s -w'

  # Create packaging tree
  TMP_PKG=\"/tmp/caterm_pkg\"
  rm -rf \"\$TMP_PKG\"
  mkdir -p \"\$TMP_PKG/usr/bin\"
  mkdir -p \"\$TMP_PKG/usr/share/applications\"
  
  cp build/bin/caterm \"\$TMP_PKG/usr/bin/\"
  chmod +x \"\$TMP_PKG/usr/bin/caterm\"

  cat << 'DESKTOP' > \"\$TMP_PKG/usr/share/applications/caterm.desktop\"
[Desktop Entry]
Name=CATerm
Comment=Zero-Knowledge SSH Connection Manager
Exec=/usr/bin/caterm
Icon=caterm
Terminal=false
Type=Application
Categories=Utility;System;Network;
DESKTOP

  # 1. Package .deb
  dpkg-deb --build deb_dir dist/caterm_1.2.0_amd64.deb 2>/dev/null || fpm -s dir -t deb -n caterm -v $VERSION -C \"\$TMP_PKG\" --prefix / -p dist/caterm__amd64.deb

  # 2. Package .rpm
  fpm -s dir -t rpm -n caterm -v $VERSION -d webkit2gtk4.1 -C \"\$TMP_PKG\" --prefix / -p dist/caterm--1.x86_64.rpm

  # 3. Package .tar.gz
  TAR_DIR=\"dist/caterm__linux_amd64\"
  rm -rf \"\$TAR_DIR\"
  mkdir -p \"\$TAR_DIR\"
  cp build/bin/caterm \"\$TAR_DIR/\"
  cat << 'README' > \"\$TAR_DIR/README.txt\"
CATerm Linux Portable Install:
1. cd caterm__linux_amd64
2. sudo cp caterm /usr/local/bin/
3. sudo chmod +x /usr/local/bin/caterm
4. caterm

Note: Requires webkit2gtk4.1 (Fedora: sudo dnf install webkit2gtk4.1 | Ubuntu/Debian: sudo apt install libwebkit2gtk-4.1-0)
README
  cd dist
  tar -czvf caterm__linux_amd64.tar.gz caterm__linux_amd64/
  rm -rf caterm__linux_amd64
"

echo "Build all completed successfully!"
