#!/usr/bin/env bash
set -euo pipefail

REPO="limistah/heimdal"
BINARY="heimdal"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and arch
OS=""
ARCH=""
case "$(uname -s)" in
    Darwin) OS="darwin" ;;
    Linux)  OS="linux"  ;;
    *)      echo "Unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac

case "$(uname -m)" in
    arm64|aarch64) ARCH="arm64" ;;
    x86_64)        ARCH="amd64" ;;
    *)             echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

ARTIFACT="${BINARY}-${OS}-${ARCH}"

# Get latest release version from GitHub API
if command -v curl >/dev/null 2>&1; then
    LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
elif command -v wget >/dev/null 2>&1; then
    LATEST=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
else
    echo "Error: curl or wget is required" >&2
    exit 1
fi

if [ -z "$LATEST" ]; then
    echo "Error: Could not determine latest version" >&2
    exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ARTIFACT}.tar.gz"

echo "Installing heimdal ${LATEST} (${OS}/${ARCH})..."

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Download
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARTIFACT}.tar.gz"
else
    wget -qO "${TMP_DIR}/${ARTIFACT}.tar.gz" "$DOWNLOAD_URL"
fi

# Extract
tar -xzf "${TMP_DIR}/${ARTIFACT}.tar.gz" -C "${TMP_DIR}"

# Install
if [ -w "$INSTALL_DIR" ]; then
    cp "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
else
    echo "Installing to ${INSTALL_DIR} requires sudo..."
    sudo cp "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY}"
fi

echo "heimdal ${LATEST} installed to ${INSTALL_DIR}/${BINARY}"
echo "Run 'heimdal --version' to verify."
