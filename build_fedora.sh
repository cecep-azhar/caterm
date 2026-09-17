#!/bin/bash
set -e

# Run build inside a debian container with webkit2gtk_4_1 tag
podman run --rm -i -v "$PWD":/app:z -w /app debian:12-slim bash -c "
  apt-get update && apt-get install -y \
    curl gcc g++ make git pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev \
    tar wget dpkg-dev
  
  # Install Node.js
  curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
  apt-get install -y nodejs
  
  # Install Go
  curl -fsSL https://go.dev/dl/go1.22.4.linux-amd64.tar.gz | tar -C /usr/local -xz
  export PATH=/usr/local/go/bin:\$PATH
  
  # Install Wails
  go install github.com/wailsapp/wails/v2/cmd/wails@latest
  export PATH=\$(go env GOPATH)/bin:\$PATH
  
  # Build with webkit2gtk_4_1 tag
  wails build -platform linux/amd64 -tags webkit2gtk_4_1 -clean -ldflags '-s -w'
"
