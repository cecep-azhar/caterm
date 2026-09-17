FROM debian:12-slim

RUN apt-get update -qq && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libwebkit2gtk-4.0-dev \
    libgtk-3-dev \
    rpm \
    && rm -rf /var/lib/apt/lists/*

# Install Go
RUN curl -L https://go.dev/dl/go1.22.4.linux-amd64.tar.gz | tar -C /usr/local -xz
ENV PATH="/usr/local/go/bin:${PATH}"

# Install Node
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

# Install Wails
RUN go install github.com/wailsapp/wails/v2/cmd/wails@v2.8.1
ENV PATH="/root/go/bin:${PATH}"

WORKDIR /app
COPY . .

# We only run the build inside the container, passing the output back out
CMD ["bash", "-c", "wails build -platform linux/amd64 && cp build/bin/caterm /out/"]
