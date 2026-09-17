#!/bin/bash
cd /home/cecepazhar/Project/caterm

echo "Running container to build..."
podman run --name caterm-extract caterm-builder bash -c "wails build -platform linux/amd64 -s"
podman cp caterm-extract:/app/build/bin/caterm ./build/bin/caterm-linux-amd64
podman rm caterm-extract
echo "Done extracting!"
