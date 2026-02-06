#!/bin/bash
# Installation script for Heimdal on Debian/Ubuntu systems

set -e

VERSION="1.0.0"
ARCH=$(dpkg --print-architecture)
DEB_FILE="heimdal_${VERSION}_${ARCH}.deb"
DOWNLOAD_URL="https://github.com/limistah/heimdal/releases/download/v${VERSION}/${DEB_FILE}"

echo "Installing Heimdal ${VERSION} for ${ARCH}..."

# Download the .deb file
curl -LO "$DOWNLOAD_URL"

# Install the package
sudo dpkg -i "$DEB_FILE"

# Clean up
rm "$DEB_FILE"

echo "Heimdal installed successfully!"
echo "Run 'heimdal --version' to verify."
