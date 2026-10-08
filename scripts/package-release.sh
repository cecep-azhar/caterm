#!/usr/bin/env bash
# ==============================================================================
# Script Otomasi Packaging & SHA-256 Manifest Generator CATerm (Tauri v2)
# ==============================================================================
set -euo pipefail

# Inisialisasi default
DIST_DIR="${1:-dist}"
VERSION="${RELEASE_VERSION:-}"
BASE_DOWNLOAD_URL="${BASE_DOWNLOAD_URL:-https://caterm.fathforce.com/dl/caterm}"
NOTES="${RELEASE_NOTES:-Release notes for CATerm}"
PUB_DATE="${RELEASE_PUB_DATE:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"

# Baca versi dari tauri.conf.json jika tidak disediakan via env
if [[ -z "$VERSION" ]]; then
  TAURI_CONF="$(dirname "$0")/../crates/caterm-app/tauri.conf.json"
  if [[ -f "$TAURI_CONF" ]]; then
    VERSION=$(grep -o '"version": *"[^"]*"' "$TAURI_CONF" | head -n1 | cut -d'"' -f4 || true)
  fi
fi

if [[ -z "$VERSION" ]]; then
  echo "Error: Version tidak ditemukan. Set RELEASE_VERSION atau pastikan tauri.conf.json valid." >&2
  exit 1
fi

# Format tag
TAG="v${VERSION#v}"

echo "==> Packaging release untuk versi: $VERSION ($TAG)"
echo "==> Direktori target: $DIST_DIR"

if [[ ! -d "$DIST_DIR" ]]; then
  echo "Direktori $DIST_DIR tidak ditemukan, membuat direktori..."
  mkdir -p "$DIST_DIR"
fi

# 1. Menghitung SHA-256 hash untuk seluruh biner (.AppImage, .deb, .tar.gz, .rpm, .exe, .apk)
echo "==> Menghitung SHA-256 hash biner..."
cd "$DIST_DIR"

FILES_TO_HASH=$(find . -maxdepth 1 -type f \( \
  -name "*.AppImage" -o \
  -name "*.deb" -o \
  -name "*.rpm" -o \
  -name "*.tar.gz" -o \
  -name "*.exe" -o \
  -name "*.msi" -o \
  -name "*.apk" -o \
  -name "*.zip" \
\) | sed 's|^\./||' | sort || true)

if [[ -n "$FILES_TO_HASH" ]]; then
  sha256sum $FILES_TO_HASH > SHA256SUMS.txt
  echo "==> SHA256SUMS.txt berhasil dibuat:"
  cat SHA256SUMS.txt
else
  echo "Perhatian: Tidak ada file biner yang ditemukan di $DIST_DIR untuk di-hash."
  touch SHA256SUMS.txt
fi

# 2. Menghasilkan manifest latest.json yang sesuai dengan format Tauri v2 updater
echo "==> Menghasilkan manifest latest.json..."

# Helper fungsi untuk membaca signature dari file .sig jika ada
get_signature() {
  local target_file="$1"
  if [[ -f "${target_file}.sig" ]]; then
    cat "${target_file}.sig" | tr -d '\r\n'
  else
    echo ""
  fi
}

PLATFORMS_JSON="{}"

# Cari artefak Linux x86_64
LINUX_FILE=""
for f in *.AppImage.tar.gz *.AppImage *.tar.gz; do
  if [[ -f "$f" ]] && [[ "$f" =~ (x86_64|amd64) ]]; then
    LINUX_FILE="$f"
    break
  fi
done

if [[ -n "$LINUX_FILE" ]]; then
  SIG=$(get_signature "$LINUX_FILE")
  URL="${BASE_DOWNLOAD_URL}/${TAG}/${LINUX_FILE}"
  PLATFORMS_JSON=$(python3 -c "
import json, sys
data = json.loads('''$PLATFORMS_JSON''')
data['linux-x86_64'] = {
    'signature': '''$SIG''',
    'url': '''$URL'''
}
print(json.dumps(data))
")
fi

# Cari artefak Windows x86_64 jika ada
WIN_FILE=""
for f in *.nsis.zip *.exe *.msi.zip; do
  if [[ -f "$f" ]] && [[ "$f" =~ (x64|x86_64) ]]; then
    WIN_FILE="$f"
    break
  fi
done

if [[ -n "$WIN_FILE" ]]; then
  SIG=$(get_signature "$WIN_FILE")
  URL="${BASE_DOWNLOAD_URL}/${TAG}/${WIN_FILE}"
  PLATFORMS_JSON=$(python3 -c "
import json, sys
data = json.loads('''$PLATFORMS_JSON''')
data['windows-x86_64'] = {
    'signature': '''$SIG''',
    'url': '''$URL'''
}
print(json.dumps(data))
")
fi

# Susun manifest latest.json akhir
python3 -c "
import json, sys

manifest = {
    'version': '$TAG',
    'notes': '''$NOTES''',
    'pub_date': '$PUB_DATE',
    'platforms': json.loads('''$PLATFORMS_JSON''')
}

with open('latest.json', 'w') as f:
    json.dump(manifest, f, indent=2)
"

echo "==> latest.json berhasil dibuat:"
cat latest.json

echo -e "\n==> Packaging & manifest generation selesai."
