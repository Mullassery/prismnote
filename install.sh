#!/bin/bash
# PrismNote Installer Script

set -e

VERSION="0.1.0"
GITHUB_REPO="Mullassery/prismnote"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect platform
detect_platform() {
    case "$(uname -s)" in
        Darwin*)
            echo "macos"
            ;;
        Linux*)
            echo "linux"
            ;;
        MINGW64*)
            echo "windows"
            ;;
        *)
            echo "unsupported"
            ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64)
            echo "x86_64"
            ;;
        arm64|aarch64)
            echo "aarch64"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

PLATFORM=$(detect_platform)
ARCH=$(detect_arch)

if [ "$PLATFORM" = "unsupported" ]; then
    echo "Error: Unsupported platform $(uname -s)"
    exit 1
fi

if [ "$PLATFORM" = "windows" ]; then
    BINARY_NAME="prismnote-windows-x86_64.exe"
else
    BINARY_NAME="prismnote-${PLATFORM}-${ARCH}"
fi

DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${BINARY_NAME}"

echo "Installing PrismNote $VERSION..."
echo "Platform: $PLATFORM, Architecture: $ARCH"
echo "Download URL: $DOWNLOAD_URL"

# Create temporary directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Download binary
echo "Downloading binary..."
curl -L -o "$TEMP_DIR/$BINARY_NAME" "$DOWNLOAD_URL"

# Make executable
chmod +x "$TEMP_DIR/$BINARY_NAME"

# Move to install directory
echo "Installing to $INSTALL_DIR..."
sudo mv "$TEMP_DIR/$BINARY_NAME" "$INSTALL_DIR/prismnote"

echo "Installation complete!"
echo "Run 'prismnote' to start."
