#!/usr/bin/env bash
# Build PrismNote binaries for all supported platforms
# Usage: ./scripts/build-release-binaries.sh [version]

set -euo pipefail

VERSION="${1:-v1.0.0}"
RELEASE_DIR="./releases/${VERSION#v}"
BINARY_NAME="prismnote"

echo "Building PrismNote binaries for version $VERSION"
echo "=================================================="

# Create release directory
mkdir -p "$RELEASE_DIR"

# Define targets
declare -A TARGETS=(
    ["aarch64-apple-darwin"]="macOS ARM64 (Apple Silicon)"
    ["x86_64-apple-darwin"]="macOS x86_64 (Intel)"
    ["aarch64-unknown-linux-gnu"]="Linux ARM64"
    ["x86_64-unknown-linux-gnu"]="Linux x86_64"
)

# Function to build a target
build_target() {
    local target=$1
    local description=$2

    echo ""
    echo "Building for $description ($target)..."

    # Check if target is installed
    if ! rustup target list | grep -q "^$target (installed)"; then
        echo "  Installing Rust target: $target"
        rustup target add "$target"
    fi

    # Build binary
    echo "  Compiling..."
    cargo build --release --target "$target"

    # Determine binary name based on OS
    local binary_path="target/$target/release/$BINARY_NAME"
    if [[ "$target" == *"-windows"* ]]; then
        binary_path="${binary_path}.exe"
    fi

    if [ ! -f "$binary_path" ]; then
        echo "  ERROR: Binary not found at $binary_path"
        return 1
    fi

    # Create archive
    local archive_name="${BINARY_NAME}-${target}.tar.gz"
    local archive_path="$RELEASE_DIR/$archive_name"

    echo "  Creating archive: $archive_name"
    cd "$(dirname "$binary_path")"
    tar -czf - "$BINARY_NAME"* > "$OLDPWD/$archive_path"
    cd "$OLDPWD"

    # Calculate SHA256
    local sha256=$(shasum -a 256 "$archive_path" | awk '{print $1}')

    echo "  SHA256: $sha256"
    echo "  Archive: $archive_path"

    # Output for Homebrew formula
    echo "$target|$sha256" >> "$RELEASE_DIR/checksums.txt"
}

# Build for each target
echo "Build targets:"
for target in "${!TARGETS[@]}"; do
    echo "  - ${TARGETS[$target]} ($target)"
done

# Check for build tools
echo ""
echo "Checking dependencies..."
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo not found. Install from https://rustup.rs/"
    exit 1
fi

if ! command -v rustup &> /dev/null; then
    echo "ERROR: rustup not found. Install from https://rustup.rs/"
    exit 1
fi

# Build all targets
failed_builds=0
for target in "${!TARGETS[@]}"; do
    if ! build_target "$target" "${TARGETS[$target]}"; then
        ((failed_builds++))
    fi
done

# Summary
echo ""
echo "=================================================="
echo "Build Summary"
echo "=================================================="

if [ -f "$RELEASE_DIR/checksums.txt" ]; then
    echo "Checksums ($RELEASE_DIR/checksums.txt):"
    cat "$RELEASE_DIR/checksums.txt"
    echo ""
fi

ls -lh "$RELEASE_DIR"/*.tar.gz 2>/dev/null || true

if [ $failed_builds -eq 0 ]; then
    echo ""
    echo "SUCCESS: All binaries built!"
    echo ""
    echo "Next steps:"
    echo "1. Create GitHub release for $VERSION"
    echo "2. Upload binaries from $RELEASE_DIR to GitHub"
    echo "3. Update Formula/prismnote.rb with checksums from $RELEASE_DIR/checksums.txt"
    echo "4. Test with: brew install --build-from-source ./Formula/prismnote.rb"
else
    echo ""
    echo "FAILURE: $failed_builds build(s) failed"
    exit 1
fi
