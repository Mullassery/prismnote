"""CLI wrapper that downloads and runs the Rust binary."""

import os
import sys
import subprocess
import platform
import urllib.request
import shutil
import json
from pathlib import Path

__version__ = "0.1.0"

GITHUB_REPO = "Mullassery/prismnote"
CACHE_DIR = Path.home() / ".prismnote" / "bin"


def get_platform_info():
    """Get platform and architecture."""
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "darwin":
        arch = "aarch64" if machine == "arm64" else "x86_64"
        return "macos", arch
    elif system == "linux":
        return "linux", machine
    elif system == "windows":
        return "windows", machine
    else:
        raise RuntimeError(f"Unsupported platform: {system}")


def get_binary_name():
    """Get the binary name for the current platform."""
    system, arch = get_platform_info()

    if system == "darwin":
        return f"prismnote-macos-{arch}"
    elif system == "linux":
        return f"prismnote-linux-{arch}"
    elif system == "windows":
        return "prismnote-windows-x86_64.exe"

    raise RuntimeError(f"Unsupported platform: {system}")


def download_binary(version: str = __version__):
    """Download the pre-built binary from GitHub Releases."""
    binary_name = get_binary_name()
    url = f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/{binary_name}"

    print(f"Downloading PrismNote {version}...")

    os.makedirs(CACHE_DIR, exist_ok=True)
    binary_path = CACHE_DIR / binary_name

    if binary_path.exists():
        print(f"Using cached binary: {binary_path}")
        return binary_path

    try:
        urllib.request.urlretrieve(url, binary_path)
        os.chmod(binary_path, 0o755)
        print(f"Downloaded to {binary_path}")
        return binary_path
    except Exception as e:
        print(f"Error downloading binary: {e}")
        print(
            f"You can manually download from: {url}"
        )
        sys.exit(1)


def main():
    """Main entry point."""
    try:
        binary_path = download_binary()

        # Open browser automatically
        import webbrowser
        webbrowser.open("http://localhost:8000")

        # Exec the binary (this replaces the Python process)
        os.execv(str(binary_path), [str(binary_path)])
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
