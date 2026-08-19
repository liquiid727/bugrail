//! Managed binary cache for the pinned `codebase-memory-mcp` release.
//!
//! Bugrail downloads the release archive for the current platform, verifies
//! its SHA-256 against the checksum published in the release's
//! `checksums.txt` (pinned in [`manifest`]), and extracts **only** the
//! binary into `<root>/bin/<version>/<platform>/`. The upstream installer is
//! never run — it would mutate Claude / Codex / OpenClaw agent configs and
//! install git hooks, which Bugrail explicitly must not do.
//!
//! Extraction reuses the same safeguard pattern as the app updater: capped
//! decompressed bytes, path-component sanitization, symlink/hardlink/device
//! rejection, unix mode preservation.

use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::code_intelligence::manifest::{
    self, Platform, MAX_ARCHIVE_BYTES, MAX_EXTRACTED_BYTES, PINNED_VERSION,
};
use crate::code_intelligence::CodeIntelError;

/// `<root>/bin/<version>/<platform>/` — layout segment per version and
/// platform so installs never overwrite each other.
pub fn bin_dir(root: &Path, platform: Platform) -> PathBuf {
    root.join("bin").join(PINNED_VERSION).join(platform.dir_name())
}

/// Path the managed binary is expected at, if installed.
pub fn installed_binary_path(root: &Path, platform: Platform) -> PathBuf {
    bin_dir(root, platform).join(manifest::binary_file_name())
}

/// The managed binary when it already exists on disk.
pub fn find_installed(root: &Path) -> Option<PathBuf> {
    let platform = Platform::current()?;
    let path = installed_binary_path(root, platform);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

/// Ensure the pinned binary is present, downloading and verifying it when
/// missing. Idempotent: an existing binary is returned as-is (its checksum
/// was verified at install time; the directory is Bugrail-owned).
pub async fn ensure_installed(root: &Path) -> Result<PathBuf, CodeIntelError> {
    let platform = Platform::current().ok_or_else(|| {
        CodeIntelError::UnsupportedPlatform(
            "codebase-memory-mcp publishes no archive for this host".into(),
        )
    })?;

    let installed = installed_binary_path(root, platform);
    if installed.is_file() {
        return Ok(installed);
    }

    let url = platform.download_url();
    tracing::info!(
        "[CodeIntel] downloading {} {} for {}",
        manifest::ADAPTER_ID,
        PINNED_VERSION,
        platform.dir_name()
    );
    let bytes = download_archive(&url).await?;
    verify_sha256(&bytes, platform.sha256())?;

    // Extract into a staging dir next to the final one, then move it into
    // place so a half-written dir is never mistaken for an install.
    let final_dir = bin_dir(root, platform);
    std::fs::create_dir_all(final_dir.parent().unwrap()).map_err(CodeIntelError::io)?;
    let staging = final_dir.with_extension("staging");
    if staging.exists() {
        std::fs::remove_dir_all(&staging).map_err(CodeIntelError::io)?;
    }
    std::fs::create_dir_all(&staging).map_err(CodeIntelError::io)?;

    let extract_result = if platform.is_zip() {
        extract_zip(&bytes, &staging, MAX_EXTRACTED_BYTES)
    } else {
        extract_tar_gz(&bytes, &staging, MAX_EXTRACTED_BYTES)
    };
    if let Err(err) = extract_result {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(err);
    }

    let staged_binary = find_binary_in(&staging)?;
    if installed.exists() {
        std::fs::remove_file(&installed).map_err(CodeIntelError::io)?;
    }
    std::fs::create_dir_all(&final_dir).map_err(CodeIntelError::io)?;
    if let Err(err) = std::fs::rename(&staged_binary, &installed) {
        // Cross-device fallback: copy then remove (staging and final dir are
        // normally on the same volume, so this is the rare path).
        tracing::debug!("[CodeIntel] rename failed ({err}); falling back to copy");
        std::fs::copy(&staged_binary, &installed).map_err(CodeIntelError::io)?;
        let _ = std::fs::remove_file(&staged_binary);
    }
    let _ = std::fs::remove_dir_all(&staging);
    ensure_executable(&installed)?;
    tracing::info!("[CodeIntel] installed {} at {}", PINNED_VERSION, installed.display());
    Ok(installed)
}

/// Locate the adapter binary somewhere inside an extracted tree (archives
/// wrap it in a single directory).
fn find_binary_in(dir: &Path) -> Result<PathBuf, CodeIntelError> {
    let name = manifest::binary_file_name();
    if dir.join(name).is_file() {
        return Ok(dir.join(name));
    }
    for entry in walkdir::WalkDir::new(dir).max_depth(3) {
        let entry = entry.map_err(|err| CodeIntelError::Extract(err.to_string()))?;
        if entry.file_type().is_file() && entry.file_name() == name {
            return Ok(entry.into_path());
        }
    }
    Err(CodeIntelError::Internal(format!(
        "archive did not contain the {name} binary"
    )))
}

async fn download_archive(url: &str) -> Result<Vec<u8>, CodeIntelError> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("bugrail-code-intelligence/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|err| CodeIntelError::Download(err.to_string()))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| CodeIntelError::Download(err.to_string()))?;
    if !response.status().is_success() {
        return Err(CodeIntelError::Download(format!(
            "release download returned HTTP {}",
            response.status()
        )));
    }
    if let Some(total) = response.content_length() {
        if total > MAX_ARCHIVE_BYTES {
            return Err(CodeIntelError::Download(format!(
                "release archive unexpectedly large ({total} bytes)"
            )));
        }
    }
    let mut downloaded: u64 = 0;
    let mut buf: Vec<u8> = Vec::new();
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| CodeIntelError::Download(err.to_string()))?;
        downloaded += chunk.len() as u64;
        if downloaded > MAX_ARCHIVE_BYTES {
            return Err(CodeIntelError::Download(
                "release archive exceeded the maximum allowed size".into(),
            ));
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

fn verify_sha256(bytes: &[u8], expected_hex: &str) -> Result<(), CodeIntelError> {
    let digest = Sha256::digest(bytes);
    let actual: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    if actual != expected_hex {
        return Err(CodeIntelError::ChecksumMismatch {
            expected: expected_hex.to_string(),
            actual,
        });
    }
    Ok(())
}

// ─── extraction (same safeguards as update/install.rs) ──────────────────

fn extract_tar_gz(bytes: &[u8], dest: &Path, max: u64) -> Result<(), CodeIntelError> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));
    let entries = archive
        .entries()
        .map_err(|err| CodeIntelError::Extract(format!("read tar entries: {err}")))?;
    let mut extracted: u64 = 0;
    for entry in entries {
        let mut entry = entry.map_err(|err| CodeIntelError::Extract(format!("tar entry: {err}")))?;
        let rel = entry
            .path()
            .map_err(|err| CodeIntelError::Extract(format!("tar entry path: {err}")))?
            .into_owned();
        let safe = sanitize_entry_path(&rel)?;
        let out = dest.join(&safe);
        let etype = entry.header().entry_type();
        if etype.is_dir() {
            std::fs::create_dir_all(&out).map_err(CodeIntelError::io)?;
        } else if etype.is_file() {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(CodeIntelError::io)?;
            }
            let remaining = max - extracted;
            #[cfg(unix)]
            let mode = entry.header().mode().ok();
            let mut out_file = std::fs::File::create(&out).map_err(CodeIntelError::io)?;
            let written = std::io::copy(&mut entry.by_ref().take(remaining + 1), &mut out_file)
                .map_err(|err| CodeIntelError::Extract(format!("unpack tar entry: {err}")))?;
            if written > remaining {
                return Err(CodeIntelError::Extract(
                    "archive decompresses to more than the allowed size".into(),
                ));
            }
            extracted += written;
            #[cfg(unix)]
            if let Some(mode) = mode {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&out, std::fs::Permissions::from_mode(mode));
            }
        } else {
            return Err(CodeIntelError::Extract(format!(
                "archive contains an unsupported entry type ({etype:?}): {}",
                safe.display()
            )));
        }
    }
    Ok(())
}

fn extract_zip(bytes: &[u8], dest: &Path, max: u64) -> Result<(), CodeIntelError> {
    use zip::ZipArchive;

    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|err| CodeIntelError::Extract(err.to_string()))?;
    let mut extracted: u64 = 0;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|err| CodeIntelError::Extract(err.to_string()))?;
        let Some(rel) = file.enclosed_name() else {
            return Err(CodeIntelError::Extract(
                "archive contains an unsafe path entry".into(),
            ));
        };
        let out = dest.join(rel);
        if file.is_dir() {
            std::fs::create_dir_all(&out).map_err(CodeIntelError::io)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(CodeIntelError::io)?;
        }
        let remaining = max - extracted;
        #[cfg(unix)]
        let mode = file.unix_mode();
        let mut writer = std::fs::File::create(&out).map_err(CodeIntelError::io)?;
        let written = std::io::copy(&mut file.by_ref().take(remaining + 1), &mut writer)
            .map_err(|err| CodeIntelError::Extract(err.to_string()))?;
        if written > remaining {
            return Err(CodeIntelError::Extract(
                "archive decompresses to more than the allowed size".into(),
            ));
        }
        extracted += written;
        #[cfg(unix)]
        if let Some(mode) = mode {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&out, std::fs::Permissions::from_mode(mode));
        }
    }
    Ok(())
}

fn sanitize_entry_path(p: &Path) -> Result<PathBuf, CodeIntelError> {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::Normal(s) => out.push(s),
            Component::CurDir => {}
            _ => {
                return Err(CodeIntelError::Extract(format!(
                    "archive contains an unsafe path entry: {}",
                    p.display()
                )))
            }
        }
    }
    Ok(out)
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<(), CodeIntelError> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(path).map_err(CodeIntelError::io)?;
    let mut perms = meta.permissions();
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(path, perms).map_err(CodeIntelError::io)
}

#[cfg(not(unix))]
fn ensure_executable(_path: &Path) -> Result<(), CodeIntelError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_verifies_and_rejects() {
        let bytes = b"hello world";
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_sha256(bytes, expected).is_ok());
        assert!(matches!(
            verify_sha256(bytes, &"0".repeat(64)),
            Err(CodeIntelError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn sanitize_rejects_traversal() {
        assert!(sanitize_entry_path(Path::new("../evil")).is_err());
        assert!(sanitize_entry_path(Path::new("/abs/path")).is_err());
        assert_eq!(
            sanitize_entry_path(Path::new("./a/b")).unwrap(),
            PathBuf::from("a/b")
        );
    }

    #[test]
    fn tar_gz_rejects_symlinks_and_bombs() {
        // Build a tar.gz containing a symlink entry in memory.
        let mut tar_bytes: Vec<u8> = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_cksum();
            builder
                .append_link(&mut header, "link", "/etc/passwd")
                .unwrap();
            builder.finish().unwrap();
        }
        let mut gz: Vec<u8> = Vec::new();
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut gz, flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, &tar_bytes).unwrap();
            encoder.finish().unwrap();
        }
        let tmp = tempfile::tempdir().unwrap();
        let err = extract_tar_gz(&gz, tmp.path(), 1024).unwrap_err();
        assert!(matches!(err, CodeIntelError::Extract(_)));
    }

    #[test]
    fn tar_gz_extracts_regular_file_and_enforces_cap() {
        let payload = vec![7u8; 4096];
        let mut tar_bytes: Vec<u8> = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            let mut header = tar::Header::new_gnu();
            header.set_size(payload.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, "dir/bin", payload.as_slice())
                .unwrap();
            builder.finish().unwrap();
        }
        let mut gz: Vec<u8> = Vec::new();
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut gz, flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, &tar_bytes).unwrap();
            encoder.finish().unwrap();
        }

        let tmp = tempfile::tempdir().unwrap();
        extract_tar_gz(&gz, tmp.path(), 8192).unwrap();
        let out = tmp.path().join("dir/bin");
        assert!(out.is_file());
        assert_eq!(std::fs::read(&out).unwrap().len(), 4096);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert!(std::fs::metadata(&out).unwrap().permissions().mode() & 0o111 != 0);
        }

        // Same archive with a cap below the payload size must fail.
        let tmp2 = tempfile::tempdir().unwrap();
        assert!(extract_tar_gz(&gz, tmp2.path(), 1024).is_err());
    }

    #[test]
    fn bin_dir_layout_is_versioned_and_platformed() {
        let root = Path::new("/data/code-intelligence/codebase-memory-mcp");
        let dir = bin_dir(root, Platform::DarwinArm64);
        assert_eq!(
            dir,
            root.join("bin")
                .join(PINNED_VERSION)
                .join("darwin-arm64")
        );
        let bin = installed_binary_path(root, Platform::DarwinArm64);
        assert!(bin.starts_with(&dir));
    }
}
