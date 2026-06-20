"""
PrismNote CLI - Downloads and launches PrismNote binary

Supports:
- macOS: M1, M2, M3, M4, M5, M6, M7, M8 (and future) Apple Silicon
- macOS: Intel x86_64
- Linux: x86_64, ARM64, ARMv7
- Windows: x86_64, ARM64
"""

import os
import sys
import subprocess
import platform
import urllib.request
from pathlib import Path

__version__ = "0.4.0"

GITHUB_REPO = "Mullassery/prismnote"
CACHE_DIR = Path.home() / ".prismnote" / "bin"


class PlatformDetector:
    """Detects OS, architecture, and Apple Silicon version"""

    @staticmethod
    def get_os():
        """Return 'macos', 'linux', or 'windows'"""
        system = platform.system().lower()
        if system == "darwin":
            return "macos"
        return system

    @staticmethod
    def get_apple_silicon_version():
        """Detect Apple Silicon M-series version"""
        if PlatformDetector.get_os() != "macos":
            return None

        try:
            # Run sysctl to get CPU subtype
            result = subprocess.run(
                ["sysctl", "-n", "hw.cpusubtype"],
                capture_output=True,
                text=True,
                timeout=5,
            )
            cpusubtype = int(result.stdout.strip())

            # Map CPU subtypes to M-series versions
            m_series_map = {
                0x01: "m1",
                0x02: "m1",
                0x03: "m2",
                0x04: "m2",
                0x05: "m3",
                0x06: "m3",
                0x07: "m4",
                0x08: "m4",
                0x09: "m5",
                0x0a: "m5",
                0x0b: "m6",  # Future M6
                0x0c: "m6",  # Future M6
                0x0d: "m7",  # Future M7
                0x0e: "m7",  # Future M7
                0x0f: "m8",  # Future M8
                0x10: "m8",  # Future M8
            }

            m_version = m_series_map.get(cpusubtype)
            if m_version:
                return m_version

            # Fallback: check if it's Apple Silicon (aarch64)
            if platform.machine() == "arm64":
                return "generic-arm64"

        except (subprocess.TimeoutExpired, ValueError, FileNotFoundError):
            # If we can't detect via sysctl, check machine type
            if platform.machine() == "arm64":
                return "generic-arm64"

        return None

    @staticmethod
    def get_arch():
        """Return architecture string"""
        machine = platform.machine().lower()

        # Handle macOS Apple Silicon detection
        os_type = PlatformDetector.get_os()
        if os_type == "macos" and machine == "arm64":
            m_version = PlatformDetector.get_apple_silicon_version()
            if m_version and m_version != "generic-arm64":
                return m_version
            return "arm64"

        # Standard architecture names
        arch_map = {
            "x86_64": "x86_64",
            "amd64": "x86_64",
            "aarch64": "aarch64",
            "arm64": "aarch64",
            "armv7l": "armv7",
            "armv7": "armv7",
        }

        return arch_map.get(machine, machine)

    @staticmethod
    def get_binary_name():
        """Get platform-specific binary name"""
        os_type = PlatformDetector.get_os()
        arch = PlatformDetector.get_arch()

        binary_map = {
            ("macos", "m1"): "prismnote-macos-m1",
            ("macos", "m2"): "prismnote-macos-m2",
            ("macos", "m3"): "prismnote-macos-m3",
            ("macos", "m4"): "prismnote-macos-m4",
            ("macos", "m5"): "prismnote-macos-m5",
            ("macos", "m6"): "prismnote-macos-m6",
            ("macos", "m7"): "prismnote-macos-m7",
            ("macos", "m8"): "prismnote-macos-m8",
            ("macos", "x86_64"): "prismnote-macos-intel",
            ("macos", "arm64"): "prismnote-macos-m1",  # Fallback for generic arm64
            ("linux", "x86_64"): "prismnote-linux-x86_64",
            ("linux", "aarch64"): "prismnote-linux-aarch64",
            ("linux", "armv7"): "prismnote-linux-armv7",
            ("windows", "x86_64"): "prismnote-windows-x86_64.exe",
            ("windows", "aarch64"): "prismnote-windows-aarch64.exe",
        }

        return binary_map.get((os_type, arch), "prismnote")

    @staticmethod
    def print_info():
        """Print detection info"""
        os_type = PlatformDetector.get_os()
        arch = PlatformDetector.get_arch()
        binary = PlatformDetector.get_binary_name()
        print(f"Platform: {os_type} {arch}", file=sys.stderr)
        print(f"Binary: {binary}", file=sys.stderr)


def get_binary_name():
    """Get the binary name for the current platform."""
    return PlatformDetector.get_binary_name()


def download_binary(version: str = __version__):
    """Download the pre-built binary from GitHub Releases."""
    binary_name = get_binary_name()
    url = f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/{binary_name}"

    print(f"Downloading PrismNote {version}...")
    PlatformDetector.print_info()

    os.makedirs(CACHE_DIR, exist_ok=True)
    binary_path = CACHE_DIR / binary_name

    if binary_path.exists() and os.access(binary_path, os.X_OK):
        print(f"Using cached binary: {binary_path}")
        return binary_path

    try:
        print(f"Downloading from: {url}")
        urllib.request.urlretrieve(url, binary_path)
        os.chmod(binary_path, 0o755)
        print(f"Downloaded to {binary_path}")
        return binary_path
    except Exception as e:
        print(f"Error downloading binary: {e}")
        print("\nSupported platforms:")
        print("  macOS: M1, M2, M3, M4, M5, M6, M7, M8, Intel x86_64")
        print("  Linux: x86_64, ARM64, ARMv7")
        print("  Windows: x86_64, ARM64")
        print(f"\nYou can manually download from: {url}")
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
