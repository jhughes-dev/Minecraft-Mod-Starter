use crate::error::{McmodError, Result};
use crate::global_config;
use crate::util::{http_get, http_get_bytes};
use colored::Colorize;
use std::path::Path;

const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/jhughes-dev/Minecraft-Mod-Starter/releases/latest";

pub fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!(
        "{}",
        format!("  Current version: {current_version}").cyan()
    );

    println!("{}", "  Checking for updates...".cyan());
    let latest_version = fetch_latest_version()?;

    if current_version == latest_version {
        println!(
            "{}",
            format!("  Already up to date (v{current_version})").green()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("  New version available: v{latest_version}").yellow()
    );

    let asset_name = get_asset_name()?;
    let download_url = fetch_asset_url(&latest_version, &asset_name)?;

    println!("{}", format!("  Downloading {asset_name}...").cyan());
    let binary = http_get_bytes(&download_url)?;

    let target = global_config::install_path()?;
    install_binary(&target, &binary)?;

    println!(
        "{}",
        format!("  Updated mcmod: v{current_version} → v{latest_version}").green()
    );

    // If running from a different location, let the user know where the binary was installed
    if let Ok(current_exe) = std::env::current_exe() {
        if let Ok(current_canon) = current_exe.canonicalize() {
            let target_matches = target
                .canonicalize()
                .map(|t| t == current_canon)
                .unwrap_or(false);
            if !target_matches {
                println!(
                    "{}",
                    format!("  Installed to: {}", target.display()).cyan()
                );
                println!(
                    "{}",
                    format!(
                        "  Note: you are running from {}",
                        current_exe.display()
                    )
                    .dimmed()
                );
            }
        }
    }

    // Warn if the install directory isn't on PATH
    if let Ok(dir) = global_config::install_dir() {
        if !global_config::is_on_path(&dir) {
            println!(
                "{}",
                format!(
                    "  Warning: {} is not on your PATH",
                    dir.display()
                )
                .yellow()
            );
        }
    }

    Ok(())
}

fn fetch_latest_version() -> Result<String> {
    let body = http_get(GITHUB_RELEASES_URL)?;
    let release: serde_json::Value = serde_json::from_str(&body)?;

    let tag = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McmodError::Other("No tag_name in release response".to_string()))?;

    // Strip leading 'v' if present
    let version = tag.strip_prefix('v').unwrap_or(tag);
    Ok(version.to_string())
}

fn fetch_asset_url(version: &str, asset_name: &str) -> Result<String> {
    let body = http_get(GITHUB_RELEASES_URL)?;
    let release: serde_json::Value = serde_json::from_str(&body)?;

    let assets = release
        .get("assets")
        .and_then(|v| v.as_array())
        .ok_or_else(|| McmodError::Other("No assets in release response".to_string()))?;

    for asset in assets {
        let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if name == asset_name {
            let url = asset
                .get("browser_download_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    McmodError::Other("No download URL for asset".to_string())
                })?;
            return Ok(url.to_string());
        }
    }

    Err(McmodError::Other(format!(
        "No release asset found matching '{asset_name}' for v{version}"
    )))
}

fn get_asset_name() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let name = match (os, arch) {
        ("linux", "x86_64") => "mcmod-linux-x86_64",
        ("macos", "x86_64") => "mcmod-macos-x86_64",
        ("macos", "aarch64") => "mcmod-macos-aarch64",
        ("windows", "x86_64") => "mcmod-windows-x86_64.exe",
        _ => {
            return Err(McmodError::Other(format!(
                "Unsupported platform: {os}/{arch}"
            )));
        }
    };

    Ok(name.to_string())
}

fn install_binary(target: &Path, new_binary: &[u8]) -> Result<()> {
    // Ensure the install directory exists
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if cfg!(unix) {
        install_binary_unix(target, new_binary)
    } else {
        install_binary_windows(target, new_binary)
    }
}

fn install_binary_unix(target: &Path, new_binary: &[u8]) -> Result<()> {
    let temp_path = target.with_extension("new");

    std::fs::write(&temp_path, new_binary)?;

    // Set executable permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // Atomic rename
    std::fs::rename(&temp_path, target)?;

    Ok(())
}

fn install_binary_windows(target: &Path, new_binary: &[u8]) -> Result<()> {
    let old_path = target.with_extension("exe.old");

    // Clean up any leftover old file from a previous update
    if old_path.exists() {
        let _ = std::fs::remove_file(&old_path);
    }

    if target.exists() {
        // Rename current exe out of the way
        std::fs::rename(target, &old_path)?;

        // Write new binary
        if let Err(e) = std::fs::write(target, new_binary) {
            // Try to restore the old binary
            let _ = std::fs::rename(&old_path, target);
            return Err(e.into());
        }

        // Clean up old binary
        let _ = std::fs::remove_file(&old_path);
    } else {
        // Target doesn't exist yet (first update without install script)
        std::fs::write(target, new_binary)?;
    }

    Ok(())
}

