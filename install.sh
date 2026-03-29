#!/usr/bin/env bash
set -euo pipefail

REPO="limistah/heimdal"
INSTALL_DIR="${HEIMDAL_INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin) PLATFORM="darwin" ;;
  linux)  PLATFORM="linux" ;;
  *)
    echo "Error: Unsupported operating system: $OS" >&2
    echo "Please install from source: cargo install heimdal" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64)        ARCH_LABEL="amd64" ;;
  aarch64|arm64) ARCH_LABEL="arm64" ;;
  *)
    echo "Error: Unsupported architecture: $ARCH" >&2
    echo "Please install from source: cargo install heimdal" >&2
    exit 1
    ;;
esac

# Get latest version
LATEST_URL="https://api.github.com/repos/${REPO}/releases/latest"
VERSION=$(curl -fsSL "$LATEST_URL" | grep '"tag_name"' | sed 's/.*"v\([^"]*\)".*/\1/')

if [ -z "$VERSION" ]; then
  echo "Error: Could not determine latest version" >&2
  exit 1
fi

TARBALL="heimdal-${PLATFORM}-${ARCH_LABEL}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${TARBALL}"

echo "Installing heimdal v${VERSION} for ${PLATFORM}/${ARCH_LABEL}..."

# Create temp dir and download
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$DOWNLOAD_URL" -o "$TMPDIR/$TARBALL"
tar -xzf "$TMPDIR/$TARBALL" -C "$TMPDIR"

# Install (try sudo if needed)
if [ -w "$INSTALL_DIR" ]; then
  install -m 755 "$TMPDIR/heimdal" "$INSTALL_DIR/heimdal"
else
  echo "Installing to $INSTALL_DIR (requires sudo)..."
  sudo install -m 755 "$TMPDIR/heimdal" "$INSTALL_DIR/heimdal"
fi

echo ""
echo "✓ heimdal installed to $INSTALL_DIR/heimdal"
heimdal --version
echo ""
echo "Get started: heimdal wizard"
