use anyhow::Result;
use std::env;

#[derive(Clone, Debug, PartialEq)]
pub enum OS {
    MacOS,
    Linux,
    Windows,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Architecture {
    AppleSiliconM1,
    AppleSiliconM2,
    AppleSiliconM3,
    AppleSiliconM4,
    AppleSiliconM5,
    AppleSiliconM6, // Future support
    AppleSiliconM7, // Future support
    AppleSiliconM8, // Future support
    IntelX86_64,
    ArmV7,
    Aarch64Generic,
}

pub struct Platform {
    pub os: OS,
    pub arch: Architecture,
}

impl Platform {
    pub fn detect() -> Result<Self> {
        let os = Self::detect_os()?;
        let arch = Self::detect_architecture()?;

        Ok(Platform { os, arch })
    }

    fn detect_os() -> Result<OS> {
        let os = env::consts::OS;
        match os {
            "macos" => Ok(OS::MacOS),
            "linux" => Ok(OS::Linux),
            "windows" => Ok(OS::Windows),
            _ => Err(anyhow::anyhow!("Unsupported OS: {}", os)),
        }
    }

    fn detect_architecture() -> Result<Architecture> {
        // First check if it's Apple Silicon
        if Self::is_apple_silicon()? {
            return Self::detect_apple_silicon_version();
        }

        // Fall back to standard arch detection
        let arch = env::consts::ARCH;
        match arch {
            "x86_64" => Ok(Architecture::IntelX86_64),
            "aarch64" => Ok(Architecture::Aarch64Generic),
            "arm" => Ok(Architecture::ArmV7),
            _ => Err(anyhow::anyhow!("Unsupported architecture: {}", arch)),
        }
    }

    fn is_apple_silicon() -> Result<bool> {
        // Check if running on macOS with Apple Silicon
        if env::consts::OS != "macos" {
            return Ok(false);
        }

        // Check using sysctl (macOS specific)
        let output = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("hw.product")
            .output();

        match output {
            Ok(out) => {
                let product = String::from_utf8_lossy(&out.stdout);
                Ok(product.contains("MacBook") || product.contains("Mac"))
            }
            Err(_) => {
                // Fallback: check if arch is aarch64 on macOS
                Ok(env::consts::ARCH == "aarch64")
            }
        }
    }

    fn detect_apple_silicon_version() -> Result<Architecture> {
        // Try to detect specific M-series chip version
        // Checking macOS version and hw.cpusubtype
        let output = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("hw.cpusubtype")
            .output();

        if let Ok(out) = output {
            let cpusubtype = String::from_utf8_lossy(&out.stdout);
            let cpusubtype_num: u32 = cpusubtype.trim().parse().unwrap_or(0);

            // CPU subtypes for Apple Silicon (from Apple documentation)
            // These values identify different M-series chips
            return Ok(match cpusubtype_num {
                0x00 => Architecture::Aarch64Generic,
                0x01 => Architecture::AppleSiliconM1,
                0x02 => Architecture::AppleSiliconM1,
                0x03 => Architecture::AppleSiliconM2,
                0x04 => Architecture::AppleSiliconM2,
                0x05 => Architecture::AppleSiliconM3,
                0x06 => Architecture::AppleSiliconM3,
                0x07 => Architecture::AppleSiliconM4,
                0x08 => Architecture::AppleSiliconM4,
                0x09 => Architecture::AppleSiliconM5,
                0x0a => Architecture::AppleSiliconM5,
                0x0b => Architecture::AppleSiliconM6,
                0x0c => Architecture::AppleSiliconM6,
                0x0d => Architecture::AppleSiliconM7,
                0x0e => Architecture::AppleSiliconM7,
                0x0f => Architecture::AppleSiliconM8,
                0x10 => Architecture::AppleSiliconM8,
                _ => {
                    // Unknown future M-series, assume generic aarch64
                    Architecture::Aarch64Generic
                }
            });
        }

        // Fallback to generic aarch64 if we can't determine specific version
        Ok(Architecture::Aarch64Generic)
    }

    pub fn binary_name(&self) -> String {
        match (&self.os, &self.arch) {
            (OS::MacOS, Architecture::AppleSiliconM1) => "prismnote-macos-m1".to_string(),
            (OS::MacOS, Architecture::AppleSiliconM2) => "prismnote-macos-m2".to_string(),
            (OS::MacOS, Architecture::AppleSiliconM3) => "prismnote-macos-m3".to_string(),
            (OS::MacOS, Architecture::AppleSiliconM4) => "prismnote-macos-m4".to_string(),
            (OS::MacOS, Architecture::AppleSiliconM5) => "prismnote-macos-m5".to_string(),
            (OS::MacOS, Architecture::AppleSiliconM6) => "prismnote-macos-m6".to_string(),
            (OS::MacOS, Architecture::AppleSiliconM7) => "prismnote-macos-m7".to_string(),
            (OS::MacOS, Architecture::AppleSiliconM8) => "prismnote-macos-m8".to_string(),
            (OS::MacOS, Architecture::IntelX86_64) => "prismnote-macos-intel".to_string(),
            (OS::Linux, Architecture::ArmV7) => "prismnote-linux-armv7".to_string(),
            (OS::Linux, Architecture::Aarch64Generic) => "prismnote-linux-aarch64".to_string(),
            (OS::Linux, Architecture::IntelX86_64) => "prismnote-linux-x86_64".to_string(),
            (OS::Windows, Architecture::IntelX86_64) => "prismnote-windows-x86_64.exe".to_string(),
            (OS::Windows, Architecture::Aarch64Generic) => "prismnote-windows-aarch64.exe".to_string(),
            _ => "prismnote".to_string(),
        }
    }

    pub fn download_url(&self, version: &str) -> String {
        let binary_name = self.binary_name();
        format!(
            "https://github.com/Mullassery/prismnote/releases/download/v{}/{}",
            version, binary_name
        )
    }

    pub fn is_apple_silicon_mac(&self) -> bool {
        matches!(
            (&self.os, &self.arch),
            (OS::MacOS, Architecture::AppleSiliconM1)
                | (OS::MacOS, Architecture::AppleSiliconM2)
                | (OS::MacOS, Architecture::AppleSiliconM3)
                | (OS::MacOS, Architecture::AppleSiliconM4)
                | (OS::MacOS, Architecture::AppleSiliconM5)
                | (OS::MacOS, Architecture::AppleSiliconM6)
                | (OS::MacOS, Architecture::AppleSiliconM7)
                | (OS::MacOS, Architecture::AppleSiliconM8)
        )
    }

    pub fn supported() -> bool {
        matches!(
            Self::detect(),
            Ok(Platform {
                os: OS::MacOS | OS::Linux,
                ..
            })
        ) || matches!(
            Self::detect(),
            Ok(Platform {
                os: OS::Windows,
                arch: Architecture::IntelX86_64 | Architecture::Aarch64Generic
            })
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect().expect("Platform detection failed");
        assert!(Platform::supported());
        println!(
            "Detected: {:?} on {:?}",
            platform.os, platform.arch
        );
    }

    #[test]
    fn test_binary_name_generation() {
        let m1_mac = Platform {
            os: OS::MacOS,
            arch: Architecture::AppleSiliconM1,
        };
        assert_eq!(m1_mac.binary_name(), "prismnote-macos-m1");

        let m5_mac = Platform {
            os: OS::MacOS,
            arch: Architecture::AppleSiliconM5,
        };
        assert_eq!(m5_mac.binary_name(), "prismnote-macos-m5");

        let intel_mac = Platform {
            os: OS::MacOS,
            arch: Architecture::IntelX86_64,
        };
        assert_eq!(intel_mac.binary_name(), "prismnote-macos-intel");
    }

    #[test]
    fn test_apple_silicon_detection() {
        let m1 = Platform {
            os: OS::MacOS,
            arch: Architecture::AppleSiliconM1,
        };
        assert!(m1.is_apple_silicon_mac());

        let m4 = Platform {
            os: OS::MacOS,
            arch: Architecture::AppleSiliconM4,
        };
        assert!(m4.is_apple_silicon_mac());

        let intel = Platform {
            os: OS::MacOS,
            arch: Architecture::IntelX86_64,
        };
        assert!(!intel.is_apple_silicon_mac());
    }
}
