#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Running container to build..."
podman run --name caterm-extract caterm-builder bash -c "wails build -platform linux/amd64 -s"
podman cp caterm-extract:/app/build/bin/caterm ./build/bin/caterm-linux-amd64
podman rm caterm-extract
echo "Done extracting!"
