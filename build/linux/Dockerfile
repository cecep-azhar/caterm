FROM debian:12 AS builder

RUN apt-get update && apt-get install -y \
    golang \
    libwebkit2gtk-4.0-dev \
    libgtk-3-dev \
    npm \
    rpm \
    dpkg-dev \
    gcc \
    pkg-config \
    git

RUN go install github.com/wailsapp/wails/v2/cmd/wails@latest

WORKDIR /app
COPY . .

RUN go mod download

RUN /root/go/bin/wails build -platform linux/amd64

WORKDIR /app/build/bin

# Package DEB
RUN mkdir -p /app/dist && \
    mkdir -p caterm_1.0.0_amd64/DEBIAN && \
    mkdir -p caterm_1.0.0_amd64/usr/bin && \
    cp caterm caterm_1.0.0_amd64/usr/bin/ && \
    echo "Package: caterm\nVersion: 1.0.0\nArchitecture: amd64\nMaintainer: CECEP <hi@cecepazhar.com>\nDescription: CATERM SSH Client\nDepends: libwebkit2gtk-4.0-37" > caterm_1.0.0_amd64/DEBIAN/control && \
    dpkg-deb --build caterm_1.0.0_amd64 && \
    mv caterm_1.0.0_amd64.deb /app/dist/

# Package RPM
RUN mkdir -p /root/rpmbuild/SOURCES /root/rpmbuild/SPECS /root/rpmbuild/RPMS /root/rpmbuild/SRPMS /root/rpmbuild/BUILD /root/rpmbuild/BUILDROOT && \
    echo -e "Name: caterm\nVersion: 1.0.0\nRelease: 1\nSummary: CATERM\nLicense: MIT\nRequires: webkit2gtk3\n\n%description\nCATERM SSH Client\n\n%install\nmkdir -p %{buildroot}/usr/bin\ncp /app/build/bin/caterm %{buildroot}/usr/bin/\n\n%files\n/usr/bin/caterm" > /root/rpmbuild/SPECS/caterm.spec && \
    rpmbuild -bb /root/rpmbuild/SPECS/caterm.spec && \
    mv /root/rpmbuild/RPMS/x86_64/*.rpm /app/dist/

CMD ["cp", "-r", "/app/dist/.", "/out/"]
