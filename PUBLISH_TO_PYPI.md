# Publishing PrismNote, StatGuard, and ClusterAudienceKit to PyPI

This guide explains how to build and publish all three projects to PyPI and create Homebrew taps.

---

## Prerequisites

### Required Tools

```bash
# Install build tools
pip install maturin build twine

# For Homebrew:
# - Homebrew (macOS/Linux): https://brew.sh/
# - Git with proper commit history
```

### Required Accounts & Credentials

1. **PyPI Account**
   - Create: https://pypi.org/account/register/
   - Enable two-factor authentication
   - Create API token: https://pypi.org/manage/account/tokens/

2. **GitHub Personal Access Token**
   - For Homebrew tap creation (push to Mullassery/homebrew-* repos)
   - Scopes needed: `repo` (full control of private repositories)

3. **Local Configuration**
   ```bash
   # Configure PyPI credentials
   mkdir -p ~/.pypirc
   # Content:
   # [distutils]
   # index-servers = pypi
   # 
   # [pypi]
   # repository = https://upload.pypi.org/legacy/
   # username = __token__
   # password = pypi-AgE... (your token)
   ```

---

## Version Management

### Current Versions (as of 2026-06-20)

| Project | Version | Status |
|---------|---------|--------|
| StatGuard | 0.1.0 | Ready |
| ClusterAudienceKit | 0.1.0 | Ready |
| PrismNote | 0.3.0 | Ready |

### Update Versions Before Release

1. **StatGuard** (`~/statguard/pyproject.toml` and `Cargo.toml`)
   ```toml
   version = "0.1.1"  # or 0.2.0 for minor, 1.0.0 for major
   ```

2. **ClusterAudienceKit** (`~/ClusterAudienceKit/pyproject.toml` and `Cargo.toml`)
   ```toml
   version = "0.1.1"
   ```

3. **PrismNote** (`~/prismnote/pyproject.toml`)
   ```toml
   version = "0.3.1"
   ```

---

## Publishing to PyPI (Option A: Test PyPI First)

### Test on TestPyPI (Recommended First)

```bash
# 1. Build StatGuard
cd ~/statguard
maturin build --release
python -m build

# 2. Upload to TestPyPI
twine upload dist/* -r testpypi

# 3. Test installation
pip install -i https://test.pypi.org/simple/ statguard

# 4. Repeat for ClusterAudienceKit and PrismNote
cd ~/ClusterAudienceKit
maturin build --release
python -m build
twine upload dist/* -r testpypi

cd ~/prismnote
maturin build --release
python -m build
twine upload dist/* -r testpypi
```

### Publish to Production PyPI

```bash
# Build each project
cd ~/statguard && maturin build --release && python -m build
cd ~/ClusterAudienceKit && maturin build --release && python -m build
cd ~/prismnote && maturin build --release && python -m build

# Upload to PyPI
twine upload ~/statguard/dist/*
twine upload ~/ClusterAudienceKit/dist/*
twine upload ~/prismnote/dist/*

# Verify
pip install --upgrade statguard clusteraudiencekit prismnote
statguard --version
python -c "import clusteraudiencekit; print(clusteraudiencekit.__version__)"
prismnote --version
```

---

## Publishing to PyPI (Option B: Using GitHub Actions)

### Setup GitHub Actions Workflow

Create `.github/workflows/publish-pypi.yml`:

```yaml
name: Publish to PyPI

on:
  release:
    types: [created]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: "3.11"
      
      - name: Install dependencies
        run: pip install maturin twine build
      
      - name: Build wheels
        run: maturin build --release
      
      - name: Publish to PyPI
        env:
          TWINE_USERNAME: __token__
          TWINE_PASSWORD: ${{ secrets.PYPI_API_TOKEN }}
        run: twine upload dist/*
```

### Trigger Publish

```bash
# Create GitHub release
gh release create v0.1.1 \
  --title "StatGuard v0.1.1" \
  --notes "Bug fixes and improvements"

# GitHub Actions automatically publishes to PyPI
```

---

## Creating Homebrew Taps

### Step 1: Create Homebrew Tap Repositories

```bash
# Create tap repos (one-time)
gh repo create homebrew-statguard \
  --public \
  --description "Homebrew tap for StatGuard"

gh repo create homebrew-clusteraudiencekit \
  --public \
  --description "Homebrew tap for ClusterAudienceKit"

gh repo create homebrew-prismnote \
  --public \
  --description "Homebrew tap for PrismNote"
```

### Step 2: Add Formulas to Each Tap

**homebrew-statguard/Formula/statguard.rb:**
```ruby
class Statguard < Formula
  desc "High-performance data quality and validation engine"
  homepage "https://github.com/Mullassery/statguard"
  url "https://files.pythonhosted.org/packages/.../statguard-0.1.1-cp313-cp313-macosx_10_12_x86_64.whl"
  sha256 "INSERT_ACTUAL_SHA256_HERE"
  license "MIT"
  
  depends_on "python@3.11"
  
  def install
    # Install as Python package
    venv = virtualenv_create(libexec, "python3.11")
    venv.pip_install "statguard==0.1.1"
    
    # Link CLI command
    bin.install_symlink libexec/"bin/statguard"
  end
  
  test do
    system bin/"statguard", "--version"
  end
end
```

**homebrew-clusteraudiencekit/Formula/clusteraudiencekit.rb:**
```ruby
class Clusteraudiencekit < Formula
  desc "High-performance audience segmentation engine"
  homepage "https://github.com/Mullassery/clusteraudiencekit"
  url "https://files.pythonhosted.org/packages/.../clusteraudiencekit-0.1.1-cp313-cp313-macosx_10_12_x86_64.whl"
  sha256 "INSERT_ACTUAL_SHA256_HERE"
  license "MIT"
  
  depends_on "python@3.11"
  
  def install
    venv = virtualenv_create(libexec, "python3.11")
    venv.pip_install "clusteraudiencekit==0.1.1"
  end
  
  test do
    system "python3", "-c", "import clusteraudiencekit; print(clusteraudiencekit.__version__)"
  end
end
```

**homebrew-prismnote/Formula/prismnote.rb:**
```ruby
class Prismnote < Formula
  desc "Modern Jupyter-compatible data science notebook"
  homepage "https://github.com/Mullassery/prismnote"
  url "https://files.pythonhosted.org/packages/.../prismnote-0.3.0-cp313-cp313-macosx_10_12_x86_64.whl"
  sha256 "INSERT_ACTUAL_SHA256_HERE"
  license "MIT"
  
  depends_on "python@3.11"
  depends_on "ipykernel"
  
  def install
    venv = virtualenv_create(libexec, "python3.11")
    venv.pip_install "prismnote==0.3.0"
    bin.install_symlink libexec/"bin/prismnote"
  end
  
  test do
    system bin/"prismnote", "--version"
  end
end
```

### Step 3: Generate SHA256 Checksums

```bash
# After publishing to PyPI, get the wheel files
pip download statguard==0.1.1 --no-deps

# Generate SHA256
sha256sum statguard-0.1.1-cp313-cp313-macosx_10_12_x86_64.whl

# Update the formula with the actual sha256
```

### Step 4: Push Formulas and Test

```bash
# Clone each tap
git clone https://github.com/Mullassery/homebrew-statguard.git
cd homebrew-statguard

# Add formula
mkdir -p Formula
# (create statguard.rb as above)

# Test locally
brew tap-new Mullassery/statguard . --force
brew install statguard

# If successful, push
git add .
git commit -m "Add statguard formula v0.1.1"
git push origin main
```

### Step 5: Install via Homebrew

```bash
# Install from custom tap
brew tap Mullassery/statguard
brew install statguard

# Verify
statguard --version
```

---

## Checklist Before Publishing

- [ ] Version numbers updated in all `pyproject.toml` and `Cargo.toml` files
- [ ] `CHANGELOG.md` updated with release notes
- [ ] Local builds successful: `maturin build --release`
- [ ] Tests pass: `cargo test` and `npm run build` (for PrismNote)
- [ ] README.md accurate and up-to-date
- [ ] Git commits properly tagged: `git tag v0.1.1`
- [ ] PyPI account created and token generated
- [ ] GitHub Personal Access Token created with `repo` scope

---

## Build Commands Reference

### StatGuard

```bash
cd ~/statguard
maturin build --release
python -m build
twine upload dist/*
```

### ClusterAudienceKit

```bash
cd ~/ClusterAudienceKit
maturin build --release
python -m build
twine upload dist/*
```

### PrismNote

```bash
cd ~/prismnote
maturin build --release
python -m build
twine upload dist/*
```

---

## Troubleshooting

### "Module not found" during build
```bash
pip install -e .
maturin develop
```

### "maturin: command not found"
```bash
pip install --upgrade maturin
```

### PyPI upload fails with 403
```bash
# Check token is valid
# Ensure __token__ username and token password in ~/.pypirc
cat ~/.pypirc
```

### Homebrew formula won't install
```bash
# Debug
brew install --verbose statguard

# Check dependencies
brew list statguard
```

---

## Success Indicators

✅ All three packages published to PyPI:
```bash
pip install statguard clusteraudiencekit prismnote
```

✅ All three available via Homebrew:
```bash
brew tap Mullassery/statguard
brew tap Mullassery/clusteraudiencekit
brew tap Mullassery/prismnote

brew install statguard clusteraudiencekit prismnote
```

✅ Versions show correctly:
```bash
statguard --version
python -c "import clusteraudiencekit; print(clusteraudiencekit.__version__)"
prismnote --version
```

---

## Next Steps

1. Review and update version numbers
2. Run local builds to verify
3. Create GitHub releases
4. Create Homebrew taps
5. Publish to PyPI
6. Update documentation with installation instructions

---

**Last Updated:** 2026-06-20
**Status:** Ready for Publishing
