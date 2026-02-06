#!/bin/sh
# Installation script for Heimdal via APK (Alpine Linux)

set -e

VERSION="1.0.0"
APK_FILE="heimdal-${VERSION}-r0.apk"
DOWNLOAD_URL="https://github.com/limistah/heimdal/releases/download/v${VERSION}/${APK_FILE}"

echo "Installing Heimdal ${VERSION} via APK..."

# Download the APK file
wget -O "$APK_FILE" "$DOWNLOAD_URL" || curl -LO "$DOWNLOAD_URL"

# Install the package (allows untrusted)
apk add --allow-untrusted "$APK_FILE"

# Clean up
rm "$APK_FILE"

echo "Heimdal installed successfully!"
echo "Run 'heimdal --version' to verify."
