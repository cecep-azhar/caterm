FROM debian:12-slim

RUN apt-get update -qq && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libwebkit2gtk-4.0-dev \
    libgtk-3-dev \
    rpm \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Install Go 1.22.4
RUN curl -L https://go.dev/dl/go1.22.4.linux-amd64.tar.gz | tar -C /usr/local -xz
ENV PATH="/usr/local/go/bin:${PATH}"

# Install Node
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

# Install Wails CLI v2.16.0
RUN go install github.com/wailsapp/wails/v2/cmd/wails@v2.16.0
ENV PATH="/root/go/bin:${PATH}"

WORKDIR /app
COPY . .

# Let's write the binary back encoded via base64, or just write it to stdout to capture it,
# but the proper way is to use a mounted volume and fix the permissions.
# First, let's fix the Dockerfile to change ownership to the local user via passed envs,
# or simply pack it as a deb/rpm package directly in the container.
# For now, let's just create the deb inside the container.
CMD ["bash", "-c", "wails build -platform linux/amd64 -s -skipbindings && cat build/bin/caterm_new | base64 > /out/caterm.b64"]
