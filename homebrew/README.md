# Homebrew Installation Guide for PrismNote

This directory contains Homebrew formula and configuration for PrismNote, enabling installation via Homebrew package manager.

## Quick Install (for Users)

Once the custom tap is published, users can install PrismNote with:

```bash
# Add the custom tap
brew tap Mullassery/prismnote

# Install PrismNote
brew install prismnote

# Verify installation
prismnote --version

# Start the server
prismnote
```

## Installation Methods Summary

PrismNote supports multiple installation methods:

| Method | Command | Platform | Notes |
|--------|---------|----------|-------|
| **Homebrew** | `brew install prismnote` | macOS, Linux | Recommended for Mac users |
| **pip** | `pip install prismnote` | Universal | Python package |
| **uv** | `uv tool install prismnote` | Universal | Fast Python tool installer |
| **curl** | `bash <(curl -fsSL ...)` | macOS, Linux | Direct binary installation |
| **Docker** | `docker run prismnote:latest` | Universal | Containerized |

## Directory Structure

```
homebrew/
├── README.md                           # This file
├── HOMEBREW_TAP_SETUP.md              # Detailed tap setup guide
├── completions/
│   ├── prismnote.bash                 # Bash shell completion
│   ├── _prismnote                     # Zsh shell completion
│   └── prismnote.fish                 # Fish shell completion
└── Formula/
    └── prismnote.rb                   # Homebrew formula (copy to tap)
```

## Usage After Installation

### Start PrismNote Server

```bash
# Default (localhost:8000)
prismnote

# Custom port
prismnote --port 3000

# Custom data directory
prismnote --data ~/my-notebooks

# Custom host
prismnote --host 0.0.0.0

# Enable debug logging
prismnote --log-level debug
```

### Shell Completions

Bash, Zsh, and Fish shell completions are automatically installed via Homebrew:

```bash
# Bash - type 'prismnote --' and press TAB
prismnote --<TAB>

# Zsh - with descriptions
prismnote --<TAB>

# Fish - with descriptions and examples
prismnote --<TAB>
```

## Development & Maintenance

### For Maintainers

See [HOMEBREW_TAP_SETUP.md](HOMEBREW_TAP_SETUP.md) for:
- Setting up the custom tap repository
- Creating GitHub releases with binaries
- Updating formula with checksums
- Publishing new versions
- CI/CD workflows

### Formula File

The `Formula/prismnote.rb` file describes:
- Where to download binaries (GitHub Releases)
- SHA256 checksums for security verification
- Platform-specific builds (macOS ARM64/x86_64, Linux ARM64/x86_64)
- Dependencies (Python 3.11)
- Installation paths and completions
- Post-installation instructions

## Common Commands

```bash
# Update Homebrew
brew update

# Check for updates
brew outdated prismnote

# Upgrade PrismNote
brew upgrade prismnote

# Show formula information
brew info prismnote

# Show installed version
brew list --versions prismnote

# Uninstall
brew uninstall prismnote

# Remove custom tap
brew untap Mullassery/prismnote
```

## Troubleshooting

### Installation Issues

**Problem:** Formula not found
```bash
# Solution: Ensure tap is added
brew tap Mullassery/prismnote
brew tap --list  # Verify tap is listed
```

**Problem:** SHA256 mismatch
```bash
# Solution: Update formula or clear cache
brew install --no-cache prismnote
```

**Problem:** Dependency missing
```bash
# Solution: Install Python dependency
brew install python@3.11
```

### Runtime Issues

**Problem:** Port already in use
```bash
# Solution: Use custom port
prismnote --port 3000
```

**Problem:** Permission denied on data directory
```bash
# Solution: Use writable directory
prismnote --data ~/Library/Application\ Support/PrismNote
```

**Problem:** Command not found
```bash
# Solution: Check Homebrew bin path
which prismnote  # Should show /usr/local/bin/prismnote
echo $PATH       # Should include /usr/local/bin
```

## Version Management

### Check Installed Version
```bash
prismnote --version
```

### Update to Latest
```bash
brew upgrade prismnote
```

### Install Specific Version
```bash
# Install v1.0.0
brew install prismnote@1.0.0

# Or downgrade
brew install prismnote@0.3.0
```

## Compatibility

### Supported Platforms
- macOS 10.13+ (Intel & Apple Silicon)
- Linux x86_64 (glibc 2.29+)
- Linux ARM64 (glibc 2.29+)

### Supported Shells
- Bash 4.0+
- Zsh 5.0+
- Fish 2.3+

### Python Requirements
- Python 3.11+ (installed automatically via Homebrew)

## Configuration

PrismNote uses TOML configuration files:

```bash
# Create config file
mkdir -p ~/.config/prismnote
cat > ~/.config/prismnote/config.toml << 'EOF'
[server]
port = 8000
host = "localhost"
log_level = "info"

[auth]
provider = "none"

[features]
enable_collaboration = true
enable_ai = true
EOF

# Start with config
prismnote --config ~/.config/prismnote/config.toml
```

## Advanced Usage

### Docker Alternative
If you prefer containerization:
```bash
docker run -p 8000:8000 -v ~/.prismnote:/root/.prismnote prismnote:latest
```

### Development Installation (from source)
```bash
git clone https://github.com/Mullassery/prismnote.git
cd prismnote
cargo build --release
./target/release/prismnote
```

## Related Documentation

- [Main README](../README.md) - Project overview
- [Installation Guide](../INSTALL.md) - All installation methods
- [Homebrew Formula Docs](https://docs.brew.sh/Formula-Cookbook)
- [GitHub Releases](https://github.com/Mullassery/prismnote/releases)

## Contributing

To contribute to the Homebrew formula:

1. Fork the tap repository
2. Make changes to `Formula/prismnote.rb`
3. Test locally: `brew install --build-from-source ./Formula/prismnote.rb`
4. Submit pull request with test results

## License

MIT License - See [LICENSE](../LICENSE)

## Support

- GitHub Issues: https://github.com/Mullassery/prismnote/issues
- Documentation: https://github.com/Mullassery/prismnote/wiki
- Discussions: https://github.com/Mullassery/prismnote/discussions
