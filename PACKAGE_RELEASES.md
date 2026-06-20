# Packaging and Releasing PrismNote, StatGuard, and ClusterAudienceKit

Complete guide for publishing all three projects to PyPI and Homebrew.

---

## Quick Start (TL;DR)

```bash
# 1. Update version numbers
# Edit: ~/statguard/pyproject.toml, ~/ClusterAudienceKit/pyproject.toml, ~/prismnote/pyproject.toml

# 2. Build all projects
cd ~/statguard && maturin build --release && python -m build
cd ~/ClusterAudienceKit && maturin build --release && python -m build
cd ~/prismnote && maturin build --release && python -m build

# 3. Test locally
pip install ~/statguard/dist/statguard-*.whl
pip install ~/ClusterAudienceKit/dist/clusteraudiencekit-*.whl
pip install ~/prismnote/dist/prismnote-*.whl

# 4. Publish to PyPI
twine upload ~/statguard/dist/*
twine upload ~/ClusterAudienceKit/dist/*
twine upload ~/prismnote/dist/*

# 5. Setup Homebrew (one-time)
brew tap Mullassery/statguard
brew tap Mullassery/clusteraudiencekit
brew tap Mullassery/prismnote
```

---

## Detailed Process

### Stage 1: Pre-Release Preparation

**1.1 Update Version Numbers**

Update all `pyproject.toml` files to use semantic versioning:

```bash
# StatGuard
~/statguard/pyproject.toml          # version = "0.1.1"
~/statguard/Cargo.toml               # version = "0.1.1"

# ClusterAudienceKit
~/ClusterAudienceKit/pyproject.toml  # version = "0.1.1"
~/ClusterAudienceKit/Cargo.toml      # version = "0.1.1"

# PrismNote
~/prismnote/pyproject.toml           # version = "0.3.1"
~/prismnote/crates/server/Cargo.toml # version = "0.3.1"
```

**1.2 Update Changelogs**

Create entries in each project's `CHANGELOG.md`:

```markdown
## [0.1.1] - 2026-06-20
### Added
- Feature X
- Feature Y

### Fixed
- Bug X
- Bug Y

### Changed
- Improvement X
```

**1.3 Test Locally**

```bash
# Ensure all tests pass
cd ~/statguard && cargo test
cd ~/ClusterAudienceKit && cargo test
cd ~/prismnote && cargo test
```

---

### Stage 2: Build Wheels

**2.1 Build StatGuard**

```bash
cd ~/statguard

# Clean previous builds
rm -rf build dist *.egg-info

# Build wheels
maturin build --release

# Verify wheel was created
ls dist/statguard-*.whl
```

**2.2 Build ClusterAudienceKit**

```bash
cd ~/ClusterAudienceKit

# Clean previous builds
rm -rf build dist *.egg-info

# Build wheels
maturin build --release

# Verify wheel was created
ls dist/clusteraudiencekit-*.whl
```

**2.3 Build PrismNote**

```bash
cd ~/prismnote

# Clean previous builds
rm -rf build dist *.egg-info

# Build wheels
maturin build --release

# Verify wheel was created
ls dist/prismnote-*.whl
```

---

### Stage 3: Local Testing (Optional but Recommended)

**3.1 Create Test Environment**

```bash
# Create virtual environment
python -m venv /tmp/test-packages
source /tmp/test-packages/bin/activate

# Install wheels locally
pip install ~/statguard/dist/statguard-*.whl
pip install ~/ClusterAudienceKit/dist/clusteraudiencekit-*.whl
pip install ~/prismnote/dist/prismnote-*.whl

# Test CLI commands
statguard --version
python -c "import clusteraudiencekit; print(clusteraudiencekit.__version__)"
prismnote --version

# Cleanup
deactivate
rm -rf /tmp/test-packages
```

---

### Stage 4: Publish to PyPI

**4.1 Setup PyPI Credentials**

Create `~/.pypirc`:

```ini
[distutils]
index-servers = pypi

[pypi]
repository = https://upload.pypi.org/legacy/
username = __token__
password = pypi-AgEIcHlwaS5vcmc... (your API token from https://pypi.org/manage/account/tokens/)
```

**4.2 Publish to Production**

```bash
# Install twine
pip install twine

# Upload each project
twine upload ~/statguard/dist/*
twine upload ~/ClusterAudienceKit/dist/*
twine upload ~/prismnote/dist/*

# Verify on PyPI
# Visit: https://pypi.org/project/statguard/
# Visit: https://pypi.org/project/clusteraudiencekit/
# Visit: https://pypi.org/project/prismnote/
```

**4.3 Verify Installation from PyPI**

```bash
# Create clean test environment
python -m venv /tmp/pypi-test
source /tmp/pypi-test/bin/activate

# Install from PyPI
pip install statguard
pip install clusteraudiencekit
pip install prismnote

# Test
statguard --version
python -c "import clusteraudiencekit"
prismnote --version

# Cleanup
deactivate
rm -rf /tmp/pypi-test
```

---

### Stage 5: Create GitHub Releases

**5.1 Tag Releases**

```bash
# StatGuard
cd ~/statguard
git tag -a v0.1.1 -m "StatGuard v0.1.1"
git push origin v0.1.1

# ClusterAudienceKit
cd ~/ClusterAudienceKit
git tag -a v0.1.1 -m "ClusterAudienceKit v0.1.1"
git push origin v0.1.1

# PrismNote
cd ~/prismnote
git tag -a v0.3.1 -m "PrismNote v0.3.1"
git push origin v0.3.1
```

**5.2 Create GitHub Releases**

```bash
# StatGuard
gh release create v0.1.1 \
  --repo Mullassery/statguard \
  --title "StatGuard v0.1.1" \
  --notes "Release notes here"

# ClusterAudienceKit
gh release create v0.1.1 \
  --repo Mullassery/clusteraudiencekit \
  --title "ClusterAudienceKit v0.1.1" \
  --notes "Release notes here"

# PrismNote
gh release create v0.3.1 \
  --repo Mullassery/prismnote \
  --title "PrismNote v0.3.1" \
  --notes "Release notes here"
```

---

### Stage 6: Create/Update Homebrew Taps

**6.1 Create Tap Repositories (One-time)**

```bash
# Create tap repos on GitHub
gh repo create homebrew-statguard --public --description "Homebrew tap for StatGuard"
gh repo create homebrew-clusteraudiencekit --public --description "Homebrew tap for ClusterAudienceKit"
gh repo create homebrew-prismnote --public --description "Homebrew tap for PrismNote"
```

**6.2 Add Formulas to Taps**

```bash
# Clone each tap
git clone https://github.com/Mullassery/homebrew-statguard.git
git clone https://github.com/Mullassery/homebrew-clusteraudiencekit.git
git clone https://github.com/Mullassery/homebrew-prismnote.git

# Copy formulas from this repo
cp ~/homebrew-formulas/statguard.rb ~/homebrew-statguard/Formula/
cp ~/homebrew-formulas/clusteraudiencekit.rb ~/homebrew-clusteraudiencekit/Formula/
cp ~/homebrew-formulas/prismnote.rb ~/homebrew-prismnote/Formula/
```

**6.3 Update SHA256 in Formulas**

```bash
# Get SHA256 for StatGuard wheel
sha256sum ~/statguard/dist/statguard-0.1.1-cp313-cp313-macosx_10_12_x86_64.whl
# Update in ~/homebrew-statguard/Formula/statguard.rb

# Get SHA256 for ClusterAudienceKit wheel
sha256sum ~/ClusterAudienceKit/dist/clusteraudiencekit-0.1.1-cp313-cp313-macosx_10_12_x86_64.whl
# Update in ~/homebrew-clusteraudiencekit/Formula/clusteraudiencekit.rb

# Get SHA256 for PrismNote wheel
sha256sum ~/prismnote/dist/prismnote-0.3.1-cp313-cp313-macosx_10_12_x86_64.whl
# Update in ~/homebrew-prismnote/Formula/prismnote.rb
```

**6.4 Push Formulas**

```bash
# Push each tap
cd ~/homebrew-statguard && git add Formula/ && git commit -m "Add statguard formula v0.1.1" && git push origin main
cd ~/homebrew-clusteraudiencekit && git add Formula/ && git commit -m "Add clusteraudiencekit formula v0.1.1" && git push origin main
cd ~/homebrew-prismnote && git add Formula/ && git commit -m "Add prismnote formula v0.3.1" && git push origin main
```

**6.5 Test Homebrew Installation**

```bash
# Install via Homebrew
brew tap Mullassery/statguard
brew install statguard

brew tap Mullassery/clusteraudiencekit
brew install clusteraudiencekit

brew tap Mullassery/prismnote
brew install prismnote

# Verify
statguard --version
python -c "import clusteraudiencekit"
prismnote --version
```

---

## Troubleshooting

### Build Issues

**"maturin: command not found"**
```bash
pip install --upgrade maturin
```

**"error: could not compile Rust project"**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
maturin build --release
```

### PyPI Issues

**"403 Forbidden" when uploading**
```bash
# Verify token is valid
cat ~/.pypirc

# Token should start with "pypi-"
# Check it hasn't expired: https://pypi.org/manage/account/tokens/
```

**"Package version already exists"**
```bash
# PyPI doesn't allow re-uploading same version
# Must increment version number and rebuild
```

### Homebrew Issues

**"Formula already installed"**
```bash
brew uninstall statguard
brew install statguard
```

**"Wrong formula from another tap"**
```bash
# Ensure you have the right tap
brew tap-info Mullassery/statguard

# May need to update: brew update
```

---

## Verification Checklist

- [ ] All projects have incremented version numbers
- [ ] CHANGELOG.md entries created for all projects
- [ ] Local tests pass: `cargo test`
- [ ] Wheels build successfully: `maturin build --release`
- [ ] Local wheel installation works
- [ ] PyPI upload successful (check pypi.org)
- [ ] `pip install` works from PyPI
- [ ] GitHub releases created with tags
- [ ] Homebrew taps created/updated
- [ ] Homebrew formulas have correct SHA256
- [ ] `brew install` works for all projects

---

## Current Status (2026-06-20)

| Project | PyPI | Homebrew | Status |
|---------|------|----------|--------|
| StatGuard | ❌ Not published | ❌ Not created | Ready |
| ClusterAudienceKit | ❌ Not published | ❌ Not created | Ready |
| PrismNote | ❌ Not published | ❌ Not created | Ready |

All three projects are **ready for release** with proper configuration files and wheel-building infrastructure in place.

---

## Next Actions

1. ✅ Updated version numbers (if needed)
2. ✅ Run `maturin build --release` for each project
3. ✅ Test wheels locally
4. ✅ Publish to PyPI with `twine`
5. ✅ Create GitHub releases
6. ✅ Setup Homebrew taps
7. ✅ Verify installations work

---

**Questions?** See PUBLISH_TO_PYPI.md for detailed step-by-step guide.
