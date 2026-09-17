#!/usr/bin/env bash
VERSION="${1:-0.1}"

echo "=============================================="
echo " Building CATerm v${VERSION} for Windows & Linux"
echo "=============================================="

echo "[1/2] Building for Windows..."
wails build -platform windows/amd64 -o "caterm-${VERSION}.exe"

echo "[2/2] Building for Linux..."
wails build -platform linux/amd64 -o "caterm-${VERSION}-linux-amd64"

echo ""
echo "[SUCCESS] Build completed. Output files:"
ls -lh build/bin/caterm-*
