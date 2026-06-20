# Homebrew Tap Setup for PrismNote

This guide explains how to set up and maintain the Homebrew tap for PrismNote.

## What is a Homebrew Tap?

A Homebrew tap is a custom package repository. Users can install from your tap with:
```bash
brew tap Mullassery/prismnote
brew install prismnote
```

## Repository Structure

The custom tap should be in a separate repository: `homebrew-prismnote`

```
homebrew-prismnote/
├── Formula/
│   └── prismnote.rb          # Main formula
├── README.md                  # Installation instructions
├── LICENSE                    # MIT License
└── .github/
    └── workflows/
        └── test.yml          # CI/CD for formula validation
```

## Step 1: Create the Custom Tap Repository

```bash
# Create new repository on GitHub: homebrew-prismnote
# Clone locally
git clone https://github.com/Mullassery/homebrew-prismnote.git
cd homebrew-prismnote

# Copy the formula
mkdir -p Formula
cp ../prismnote/Formula/prismnote.rb Formula/

# Create README
cat > README.md << 'EOF'
# Homebrew PrismNote Tap

Custom Homebrew tap for installing PrismNote.

## Installation

```bash
brew tap Mullassery/prismnote
brew install prismnote
```

## Usage

```bash
prismnote                    # Start server on http://localhost:8000
prismnote --port 3000        # Custom port
prismnote --data /custom/dir # Custom data directory
```

## Updating

```bash
brew upgrade prismnote
```

## Uninstalling

```bash
brew uninstall prismnote
brew untap Mullassery/prismnote
```

## Documentation

See [PrismNote GitHub](https://github.com/Mullassery/prismnote)
EOF

git add Formula/ README.md
git commit -m "Initial homebrew tap setup"
git push origin main
```

## Step 2: Create GitHub Release with Binaries

Before publishing the tap, ensure binaries are available:

```bash
# In main prismnote repository
# Build binaries for multiple platforms:

# macOS ARM64 (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# macOS x86_64 (Intel)
cargo build --release --target x86_64-apple-darwin

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# Create release archives and upload to GitHub Releases
# Name format: prismnote-{arch}-{os}.tar.gz
```

## Step 3: Update Formula with SHA256 Checksums

Calculate SHA256 for each binary:

```bash
# After uploading binaries to GitHub Releases
curl -L https://github.com/Mullassery/prismnote/releases/download/v1.0.0/prismnote-aarch64-apple-darwin.tar.gz | shasum -a 256

# Update these values in Formula/prismnote.rb:
# - PLACEHOLDER_ARM64_SHA256
# - PLACEHOLDER_X86_64_SHA256
# - PLACEHOLDER_ARM64_LINUX_SHA256
# - PLACEHOLDER_X86_64_LINUX_SHA256
```

## Step 4: Validate the Formula

```bash
# Install from local tap
brew install --build-from-source ./Formula/prismnote.rb

# Test installation
prismnote --version

# Audit formula
brew audit --new-formula Formula/prismnote.rb
```

## Step 5: Create CI/CD Workflow

Create `.github/workflows/test.yml`:

```yaml
name: Homebrew Formula Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-formula:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Validate formula syntax
        run: |
          brew install ruby
          brew audit --new-formula Formula/prismnote.rb
      - name: Test installation
        run: |
          brew install --build-from-source ./Formula/prismnote.rb
          prismnote --version
```

## Installation Instructions for Users

Once the tap is published, users can install with:

```bash
# Add the custom tap
brew tap Mullassery/prismnote

# Install PrismNote
brew install prismnote

# Verify installation
prismnote --version

# Start server
prismnote
```

## Updating the Formula

When releasing a new version:

1. Update version in `Formula/prismnote.rb`
2. Create GitHub release with new binaries
3. Calculate new SHA256 checksums
4. Update all SHA256 values in formula
5. Test with `brew upgrade prismnote`
6. Commit and push changes

## Example: Updating to v1.1.0

```ruby
# In Formula/prismnote.rb

on_macos do
  on_arm do
    url "https://github.com/Mullassery/prismnote/releases/download/v1.1.0/prismnote-aarch64-apple-darwin.tar.gz"
    sha256 "NEW_SHA256_HERE"
  end
  # ... etc
end
```

## Submitting to Homebrew Core

If you want PrismNote in the official Homebrew Core repository:

1. Ensure formula meets all requirements
2. Submit PR to homebrew/homebrew-core
3. Include:
   - Working formula file
   - GitHub release with binaries
   - Test results
   - Documentation link

## Troubleshooting

### Formula won't install
```bash
brew doctor                    # Check system
brew audit Formula/prismnote.rb # Validate syntax
```

### Wrong binary downloaded
```bash
# Check available URLs in formula
curl -I https://github.com/Mullassery/prismnote/releases/download/v1.0.0/prismnote-aarch64-apple-darwin.tar.gz
```

### SHA256 mismatch
```bash
# Recalculate checksum
curl -L https://github.com/...prismnote-aarch64-apple-darwin.tar.gz | shasum -a 256
# Update in formula
```

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Creating a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Bottle Documentation](https://docs.brew.sh/Bottles)
