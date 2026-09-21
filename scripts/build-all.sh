#!/usr/bin/env bash
set -euo pipefail

# CATerm v2 - Simple Build All Script (Linux deb/rpm/appimage & Windows x64)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

TARGET="${1:-all}"

echo "========================================"
echo "  CATerm v2 Multi-Platform Builder"
echo "  Target: $TARGET"
echo "========================================"

mkdir -p "$DIST_DIR"

# 1. Build Frontend SvelteKit
echo "--> [1/3] Building SvelteKit Frontend..."
cd "$ROOT_DIR/frontend"
npm install
npm run build

# 2. Build Linux Bundles (deb, rpm, AppImage)
if [[ "$TARGET" == "all" || "$TARGET" == "linux" ]]; then
  echo "--> [2/3] Building Linux Bundles (.deb, .rpm, .AppImage) via Podman..."
  cd "$ROOT_DIR"
  podman run --rm -v "$ROOT_DIR":/app:z -w /app rust:latest bash -c "
    set -e
    apt-get update -qq
    apt-get install -y -qq nodejs npm libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev rpm
    npm install -g @tauri-apps/cli
    cd /app/crates/caterm-app
    npx tauri build
  "

  # Copy Linux artifacts to dist/
  cp -f "$ROOT_DIR"/target/release/bundle/deb/*.deb "$DIST_DIR/" 2>/dev/null || true
  cp -f "$ROOT_DIR"/target/release/bundle/rpm/*.rpm "$DIST_DIR/" 2>/dev/null || true
  cp -f "$ROOT_DIR"/target/release/bundle/appimage/*.AppImage "$DIST_DIR/" 2>/dev/null || true
fi

# 3. Build Windows x64 (.exe)
if [[ "$TARGET" == "all" || "$TARGET" == "win" || "$TARGET" == "windows" ]]; then
  echo "--> [3/3] Cross-compiling Windows x64 (.exe) via Podman..."
  cd "$ROOT_DIR"
  podman run --rm -v "$ROOT_DIR":/app:z -w /app rust:latest bash -c "
    set -e
    apt-get update -qq
    apt-get install -y -qq mingw-w64 pkg-config
    rustup target add x86_64-pc-windows-gnu
    cargo build --target x86_64-pc-windows-gnu -p caterm-app --release
  "

  # Copy Windows artifact to dist/
  cp -f "$ROOT_DIR/target/x86_64-pc-windows-gnu/release/caterm-app.exe" "$DIST_DIR/CATerm-x64.exe" 2>/dev/null || true
fi

echo ""
echo "========================================"
echo "  Build Completed! Artifacts in dist/:"
echo "========================================"
ls -lh "$DIST_DIR"
