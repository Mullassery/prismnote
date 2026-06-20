#!/bin/bash
# Build script for macOS - supports M1, M2, M3, M4, M5, and Intel

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_DIR="$PROJECT_DIR/target/release"
GITHUB_RELEASE_DIR="$PROJECT_DIR/releases"

echo "=== PrismNote macOS Build ==="
echo "Building for Apple Silicon (M1-M8) and Intel"

# Create releases directory
mkdir -p "$GITHUB_RELEASE_DIR"

# Detect current architecture
CURRENT_ARCH=$(uname -m)
echo "Current architecture: $CURRENT_ARCH"

# Function to build for a specific target
build_for_target() {
    local target=$1
    local binary_name=$2
    local min_version=$3

    echo ""
    echo "Building for $target (minimum: macOS $min_version)..."

    MACOSX_DEPLOYMENT_TARGET=$min_version cargo build --release --target "$target"

    local binary_src="$PROJECT_DIR/target/$target/release/prismnote"
    local binary_dst="$GITHUB_RELEASE_DIR/$binary_name"

    if [ -f "$binary_src" ]; then
        cp "$binary_src" "$binary_dst"
        chmod +x "$binary_dst"
        echo "✓ Built: $binary_name"
    else
        echo "✗ Build failed for $target"
        return 1
    fi
}

# Build for Apple Silicon (M1, M2, M3, M4, M5+)
echo ""
echo "=== Building for Apple Silicon (M1, M2, M3, M4, M5, M6, M7, M8) ==="
build_for_target "aarch64-apple-darwin" "prismnote-macos-arm64" "11.0"

# Build for Intel Mac
echo ""
echo "=== Building for Intel Mac ==="
build_for_target "x86_64-apple-darwin" "prismnote-macos-intel" "10.7"

echo ""
echo "=== Build Complete ==="
echo "Binaries created in: $GITHUB_RELEASE_DIR"
ls -lh "$GITHUB_RELEASE_DIR"

echo ""
echo "=== Note on Apple Silicon Versions ==="
echo "The ARM64 binary (prismnote-macos-arm64) runs natively on:"
echo "  • MacBook Pro 13/14/15 (M1, M2, M3, M4, M5, etc.)"
echo "  • MacBook Air 13 (M1, M2, M3, M4, M5, etc.)"
echo "  • Mac mini (M1, M2, M3, M4, M5, etc.)"
echo "  • Mac Studio (M1 Max, M1 Ultra, M2 Max, M2 Ultra, etc.)"
echo "  • iMac 24 (M1, M3, M4, M5, etc.)"
echo ""
echo "Python CLI will auto-detect M-series version and download correct binary."
echo "Currently supports: M1, M2, M3, M4, M5, M6, M7, M8 (future versions supported)"
