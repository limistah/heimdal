#!/bin/bash
# Installation script for Heimdal via RPM (Fedora/RHEL/CentOS)

set -e

VERSION="1.0.0"
RPM_FILE="heimdal-${VERSION}-1.x86_64.rpm"
DOWNLOAD_URL="https://github.com/limistah/heimdal/releases/download/v${VERSION}/${RPM_FILE}"

echo "Installing Heimdal ${VERSION} via RPM..."

# Download the RPM file
curl -LO "$DOWNLOAD_URL"

# Install the package
sudo rpm -i "$RPM_FILE"

# Clean up
rm "$RPM_FILE"

echo "Heimdal installed successfully!"
echo "Run 'heimdal --version' to verify."
