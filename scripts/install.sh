#!/usr/bin/env bash
set -euo pipefail

REPO="cirbinius/cirbinius"
VERSION="${1:-latest}"
BIN_DIR="${CIRBINIUS_BIN_DIR:-/usr/local/bin}"

detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "arm64" ;;
        *) echo "unsupported-arch: $arch" >&2; exit 1 ;;
    esac
}

detect_os() {
    local os
    os="$(uname -s)"
    case "$os" in
        Linux) echo "linux" ;;
        Darwin) echo "macos" ;;
        *) echo "unsupported-os: $os" >&2; exit 1 ;;
    esac
}

main() {
    local os arch url
    os="$(detect_os)"
    arch="$(detect_arch)"
    
    if [ "$VERSION" = "latest" ]; then
        url="https://github.com/$REPO/releases/latest/download/cirbinius-${os}-${arch}.tar.gz"
    else
        url="https://github.com/$REPO/releases/download/v${VERSION}/cirbinius-${os}-${arch}.tar.gz"
    fi

    echo "Installing CirBinius v${VERSION} (${os}-${arch})..."
    
    local tmpdir
    tmpdir="$(mktemp -d)"
    cd "$tmpdir"
    
    curl -sSfL "$url" -o cirbinius.tar.gz
    echo "Downloaded $url"
    
    # Verify checksum if available
    if curl -sSfL "${url}.sha256" -o cirbinius.tar.gz.sha256 2>/dev/null; then
        sha256sum -c cirbinius.tar.gz.sha256
    fi
    
    tar xzf cirbinius.tar.gz
    
    install -m 755 cirbinius-api "$BIN_DIR/cirbinius-api"
    install -m 755 cirbinius-sdk "$BIN_DIR/cirbinius" 2>/dev/null || true
    
    cd /
    rm -rf "$tmpdir"
    
    echo "CirBinius installed to $BIN_DIR"
    echo "Run 'cirbinius doctor' to verify the installation."
}

main
