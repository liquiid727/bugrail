use std::path::{Path, PathBuf};

pub struct ProductManifest {
    pub display_name: &'static str,
    pub bundle_name: &'static str,
    pub bundle_identifier: &'static str,
    pub data_dir_name: &'static str,
    pub platform_data_dir_name: &'static str,
    pub server_fallback_data_dir_name: &'static str,
    pub keyring_service_name: &'static str,
    pub repository_slug: &'static str,
    pub update_manifest_url: &'static str,
    pub release_download_base: &'static str,
    pub updater_public_key: &'static str,
}

pub const PRODUCT_MANIFEST: ProductManifest = ProductManifest {
    display_name: "Bugrail",
    bundle_name: "Bugrail",
    bundle_identifier: "io.liquiid.bugrail",
    data_dir_name: ".bugrail",
    platform_data_dir_name: "bugrail",
    server_fallback_data_dir_name: ".bugrail-data",
    keyring_service_name: "bugrail",
    repository_slug: "liquiid727/bugrail",
    update_manifest_url:
        "https://github.com/liquiid727/bugrail/releases/latest/download/latest.json",
    release_download_base: "https://github.com/liquiid727/bugrail/releases/latest/download",
    updater_public_key: "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEYzQzc5Q0M1ODM1M0MyRUMKUldUc3dsT0R4WnpIOCtNSFUwRVlrSlJ4VGdvN3VTZWJteDB6L1RwWjZlU1FvRUpVNVFPUFZKNnkK",
};

/// Tauri dev launches the compatibility binary directly rather than through a
/// macOS app bundle. AppKit therefore falls back to the executable name and a
/// generic/cached icon unless we supply the product identity at runtime.
#[cfg(all(feature = "tauri-runtime", target_os = "macos"))]
pub fn apply_macos_runtime_identity() -> Result<(), String> {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::{NSData, NSProcessInfo, NSString};

    let main_thread = MainThreadMarker::new()
        .ok_or_else(|| "macOS product identity must be applied on the main thread".to_string())?;

    let product_name = NSString::from_str(PRODUCT_MANIFEST.display_name);
    NSProcessInfo::processInfo().setProcessName(&product_name);

    let icon_data = NSData::with_bytes(include_bytes!("../../icons/icon.png"));
    let icon = NSImage::initWithData(NSImage::alloc(), &icon_data)
        .ok_or_else(|| "failed to decode the embedded Bugrail app icon".to_string())?;
    let application = NSApplication::sharedApplication(main_thread);
    // SAFETY: `icon` is a valid, retained NSImage and AppKit accepts it as the
    // process-wide application icon. This runs on the main thread in setup.
    unsafe {
        application.setApplicationIconImage(Some(&icon));
    }

    Ok(())
}

pub fn platform_data_dir(base: &Path) -> PathBuf {
    base.join(PRODUCT_MANIFEST.platform_data_dir_name)
}

pub fn platform_cache_dir(base: &Path) -> PathBuf {
    base.join(PRODUCT_MANIFEST.platform_data_dir_name)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{platform_cache_dir, platform_data_dir, PRODUCT_MANIFEST};

    #[test]
    fn bugrail_owns_external_product_identity() {
        assert_eq!(PRODUCT_MANIFEST.display_name, "Bugrail");
        assert_eq!(PRODUCT_MANIFEST.bundle_identifier, "io.liquiid.bugrail");
        assert_eq!(PRODUCT_MANIFEST.data_dir_name, ".bugrail");
        assert_eq!(PRODUCT_MANIFEST.keyring_service_name, "bugrail");
        assert_eq!(PRODUCT_MANIFEST.repository_slug, "liquiid727/bugrail");
    }

    #[test]
    fn bugrail_update_channel_never_points_at_upstream_codeg() {
        assert!(PRODUCT_MANIFEST
            .update_manifest_url
            .starts_with("https://github.com/liquiid727/bugrail/releases/"));
        assert!(!PRODUCT_MANIFEST
            .update_manifest_url
            .contains("xintaofei/codeg"));
    }

    #[test]
    fn platform_storage_roots_use_the_bugrail_namespace() {
        let base = Path::new("/tmp/platform-root");
        assert_eq!(
            platform_data_dir(base),
            Path::new("/tmp/platform-root/bugrail")
        );
        assert_eq!(
            platform_cache_dir(base),
            Path::new("/tmp/platform-root/bugrail")
        );
    }

    #[test]
    fn tauri_bundle_identity_matches_the_product_manifest() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../../tauri.conf.json")).unwrap();
        assert_eq!(config["productName"], PRODUCT_MANIFEST.bundle_name);
        assert_eq!(config["identifier"], PRODUCT_MANIFEST.bundle_identifier);
        assert_eq!(
            config["plugins"]["updater"]["endpoints"][0],
            PRODUCT_MANIFEST.update_manifest_url
        );
        assert_eq!(
            config["plugins"]["updater"]["pubkey"],
            PRODUCT_MANIFEST.updater_public_key
        );
    }
}
