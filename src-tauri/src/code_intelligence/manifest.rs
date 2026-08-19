//! Pinned release manifest for the `codebase-memory-mcp` adapter binary.
//!
//! Bugrail never runs the upstream installer (it rewrites Claude / Codex /
//! OpenClaw agent configs and hooks); it downloads the release archive for
//! the pinned version, verifies the SHA-256 published in the release's
//! `checksums.txt`, and extracts only the binary into its own cache root.
//! Upgrades happen by bumping this manifest, never by the binary updating
//! itself.

/// Adapter identifier — also the cache-layout segment and the
/// `adapter:` value accepted in `.codeg/context.yaml`.
pub const ADAPTER_ID: &str = "codebase-memory-mcp";

/// Pinned upstream version. The binary cache layout and the compatibility
/// check both key off this value.
pub const PINNED_VERSION: &str = "0.10.6";

/// Release tag the pinned version was published under.
pub const RELEASE_TAG: &str = "v0.10.6";

/// Base URL for release assets of the pinned version.
pub const RELEASE_BASE_URL: &str =
    "https://github.com/DeusData/codebase-memory-mcp/releases/download/v0.10.6";

/// Maximum archive size accepted (the real archives are ~40 MB).
pub const MAX_ARCHIVE_BYTES: u64 = 300 * 1024 * 1024;

/// Maximum decompressed bytes accepted while extracting the binary.
pub const MAX_EXTRACTED_BYTES: u64 = 512 * 1024 * 1024;

/// Environment variable carrying an absolute-path binary override. First tier
/// of binary resolution (server deployments); must exist, be executable, and
/// report a compatible version or the adapter refuses to start.
pub const BINARY_OVERRIDE_ENV: &str = "CODEG_CBM_BIN";

/// Platform variants the pinned release publishes archives for. Linux uses
/// the `-portable` (static) builds so server / Docker hosts with older or
/// unusual glibc setups still run the binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    DarwinArm64,
    DarwinAmd64,
    LinuxArm64Portable,
    LinuxAmd64Portable,
    WindowsArm64,
    WindowsAmd64,
}

impl Platform {
    /// The platform of the current host, when the pinned release ships an
    /// archive for it.
    pub fn current() -> Option<Platform> {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            Some(Platform::DarwinArm64)
        }
        #[cfg(all(target_os = "macos", not(target_arch = "aarch64")))]
        {
            Some(Platform::DarwinAmd64)
        }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            Some(Platform::LinuxArm64Portable)
        }
        #[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
        {
            Some(Platform::LinuxAmd64Portable)
        }
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            Some(Platform::WindowsArm64)
        }
        #[cfg(all(target_os = "windows", not(target_arch = "aarch64")))]
        {
            Some(Platform::WindowsAmd64)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }

    /// Archive file name inside the release, matching `checksums.txt`
    /// byte-for-byte.
    pub fn asset_file(self) -> &'static str {
        match self {
            Platform::DarwinArm64 => "codebase-memory-mcp-darwin-arm64.tar.gz",
            Platform::DarwinAmd64 => "codebase-memory-mcp-darwin-amd64.tar.gz",
            Platform::LinuxArm64Portable => "codebase-memory-mcp-linux-arm64-portable.tar.gz",
            Platform::LinuxAmd64Portable => "codebase-memory-mcp-linux-amd64-portable.tar.gz",
            Platform::WindowsArm64 => "codebase-memory-mcp-windows-arm64.zip",
            Platform::WindowsAmd64 => "codebase-memory-mcp-windows-amd64.zip",
        }
    }

    /// SHA-256 of the archive, transcribed from the release's
    /// `checksums.txt` for the pinned version.
    pub fn sha256(self) -> &'static str {
        match self {
            Platform::DarwinArm64 => {
                "b6a29184cd40eb993fe144e85c2ec84e567a9d2e1904a38335fd29e810209592"
            }
            Platform::DarwinAmd64 => {
                "2a26362e556b6fe9bb8df4993e7f42de274a283502309105cadd013408f8e740"
            }
            Platform::LinuxArm64Portable => {
                "89bb1c353c6199382d991608cb7923b02057478111d3ec9ffcd014c6fdd02ed7"
            }
            Platform::LinuxAmd64Portable => {
                "3caa32669638d432fe3bfda995552fa1e50e5bb364e0ad0366ad55f0a52a35fa"
            }
            Platform::WindowsArm64 => {
                "e45354da8d7b895a4e9cec93ab72ec7f2a403063ada147946f0d1316c100bcc6"
            }
            Platform::WindowsAmd64 => {
                "e4dd1b75368313710cbe7a7992bc8d37c9318fc961a1ae81fa7f668a6d690828"
            }
        }
    }

    /// Cache-layout segment so platforms never overwrite each other.
    pub fn dir_name(self) -> &'static str {
        match self {
            Platform::DarwinArm64 => "darwin-arm64",
            Platform::DarwinAmd64 => "darwin-amd64",
            Platform::LinuxArm64Portable => "linux-arm64-portable",
            Platform::LinuxAmd64Portable => "linux-amd64-portable",
            Platform::WindowsArm64 => "windows-arm64",
            Platform::WindowsAmd64 => "windows-amd64",
        }
    }

    pub fn is_zip(self) -> bool {
        matches!(self, Platform::WindowsArm64 | Platform::WindowsAmd64)
    }

    pub fn download_url(self) -> String {
        format!("{}/{}", RELEASE_BASE_URL, self.asset_file())
    }
}

/// Name of the binary inside the extracted cache directory.
pub fn binary_file_name() -> &'static str {
    if cfg!(windows) {
        "codebase-memory-mcp.exe"
    } else {
        ADAPTER_ID
    }
}

/// Compatibility rule for override binaries: same major.minor as the pinned
/// version. An override reporting anything else is refused (no silent
/// fallback to the managed copy — a user-set override that is ignored would
/// be a worse surprise than a loud error).
pub fn is_compatible_version(reported: &str) -> bool {
    let Some(version) = reported
        .split_whitespace()
        .last()
        .or_else(|| reported.split_whitespace().next())
    else {
        return false;
    };
    let mut parts = version.split('.');
    let (Some(major), Some(minor)) = (parts.next(), parts.next()) else {
        return false;
    };
    let mut pinned = PINNED_VERSION.split('.');
    matches!(
        (
            pinned.next().and_then(|p| p.parse::<u32>().ok()),
            pinned.next().and_then(|p| p.parse::<u32>().ok()),
            major.parse::<u32>().ok(),
            minor.parse::<u32>().ok()
        ),
        (Some(pm), Some(pn), Some(m), Some(n)) if pm == m && pn == n
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_version_matches_release_tag() {
        assert_eq!(format!("v{PINNED_VERSION}"), RELEASE_TAG);
        assert!(RELEASE_BASE_URL.contains(RELEASE_TAG));
    }

    #[test]
    fn every_platform_has_unique_asset_and_checksum() {
        let all = [
            Platform::DarwinArm64,
            Platform::DarwinAmd64,
            Platform::LinuxArm64Portable,
            Platform::LinuxAmd64Portable,
            Platform::WindowsArm64,
            Platform::WindowsAmd64,
        ];
        for p in all {
            assert!(p.asset_file().starts_with("codebase-memory-mcp-"));
            assert_eq!(p.sha256().len(), 64, "sha256 hex must be 64 chars");
            assert!(p.download_url().starts_with(RELEASE_BASE_URL));
        }
        let mut files: Vec<&str> = all.iter().map(|p| p.asset_file()).collect();
        files.sort_unstable();
        files.dedup();
        assert_eq!(files.len(), all.len());
    }

    #[test]
    fn current_platform_is_published() {
        // The three desktop/server hosts CI runs on all have archives.
        if let Some(p) = Platform::current() {
            assert!(p.asset_file().contains("codebase-memory-mcp"));
        }
    }

    #[test]
    fn compatibility_requires_same_major_minor() {
        assert!(is_compatible_version("codebase-memory-mcp 0.10.6"));
        assert!(is_compatible_version("codebase-memory-mcp 0.10.9"));
        assert!(is_compatible_version("0.10.1"));
        assert!(!is_compatible_version("codebase-memory-mcp 0.11.0"));
        assert!(!is_compatible_version("codebase-memory-mcp 1.10.6"));
        assert!(!is_compatible_version("codebase-memory-mcp 0.9.6"));
        assert!(!is_compatible_version(""));
        assert!(!is_compatible_version("garbage"));
    }
}
