FROM debian:12-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    git \
    build-essential \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libwebkit2gtk-4.1-dev \
    npm \
    pkg-config \
    rpm \
    dpkg-dev

RUN curl -OL https://go.dev/dl/go1.24.1.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf go1.24.1.linux-amd64.tar.gz
ENV PATH=$PATH:/usr/local/go/bin

RUN go install github.com/wailsapp/wails/v2/cmd/wails@latest

WORKDIR /app
COPY . .

# Use sed to fix wails.json to output the webkit2gtk_4_1 tag explicitly if needed, but we provide BOTH dev libraries.
RUN /root/go/bin/wails build -platform linux/amd64

WORKDIR /app/build/bin

# Package DEB
RUN mkdir -p /app/dist && \
    mkdir -p caterm_1.0.0_amd64/DEBIAN && \
    mkdir -p caterm_1.0.0_amd64/usr/bin && \
    cp caterm caterm_1.0.0_amd64/usr/bin/ && \
    printf "Package: caterm\nVersion: 1.0.0\nArchitecture: amd64\nMaintainer: CECEP <hi@cecepazhar.com>\nDescription: CATERM SSH Client\nDepends: libwebkit2gtk-4.1-0 | libwebkit2gtk-4.0-37\n" > caterm_1.0.0_amd64/DEBIAN/control && \
    dpkg-deb --build caterm_1.0.0_amd64 && \
    mv caterm_1.0.0_amd64.deb /app/dist/

# Package RPM
RUN mkdir -p /root/rpmbuild/SOURCES /root/rpmbuild/SPECS /root/rpmbuild/RPMS /root/rpmbuild/SRPMS /root/rpmbuild/BUILD /root/rpmbuild/BUILDROOT && \
    printf "Name: caterm\nVersion: 1.0.0\nRelease: 1\nSummary: CATERM\nLicense: MIT\nRequires: webkit2gtk4.1\n\n%%description\nCATERM SSH Client\n\n%%install\nmkdir -p %%{buildroot}/usr/bin\ncp /app/build/bin/caterm %%{buildroot}/usr/bin/\n\n%%files\n/usr/bin/caterm\n" > /root/rpmbuild/SPECS/caterm.spec && \
    rpmbuild -bb /root/rpmbuild/SPECS/caterm.spec && \
    mv /root/rpmbuild/RPMS/x86_64/*.rpm /app/dist/

CMD ["cp", "-r", "/app/dist/.", "/out/"]
