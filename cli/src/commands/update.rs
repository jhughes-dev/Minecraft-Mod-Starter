use crate::error::{McmodError, Result};
use colored::Colorize;
use std::io::Read;

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

    replace_binary(&binary)?;

    println!(
        "{}",
        format!("  Updated mcmod: v{current_version} → v{latest_version}").green()
    );

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

fn replace_binary(new_binary: &[u8]) -> Result<()> {
    let current_exe = std::env::current_exe()
        .map_err(|e| McmodError::Other(format!("Cannot determine current exe path: {e}")))?;

    if cfg!(unix) {
        replace_binary_unix(&current_exe, new_binary)
    } else {
        replace_binary_windows(&current_exe, new_binary)
    }
}

fn replace_binary_unix(current_exe: &std::path::Path, new_binary: &[u8]) -> Result<()> {
    let temp_path = current_exe.with_extension("new");

    std::fs::write(&temp_path, new_binary)?;

    // Set executable permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // Atomic rename
    std::fs::rename(&temp_path, current_exe)?;

    Ok(())
}

fn replace_binary_windows(current_exe: &std::path::Path, new_binary: &[u8]) -> Result<()> {
    let old_path = current_exe.with_extension("exe.old");

    // Clean up any leftover old file from a previous update
    if old_path.exists() {
        let _ = std::fs::remove_file(&old_path);
    }

    // Rename current exe out of the way
    std::fs::rename(current_exe, &old_path)?;

    // Write new binary
    if let Err(e) = std::fs::write(current_exe, new_binary) {
        // Try to restore the old binary
        let _ = std::fs::rename(&old_path, current_exe);
        return Err(e.into());
    }

    // Clean up old binary
    let _ = std::fs::remove_file(&old_path);

    Ok(())
}

fn http_get(url: &str) -> Result<String> {
    let body = ureq::get(url)
        .header("User-Agent", "mcmod-cli")
        .call()
        .map_err(|e| McmodError::Http(format!("{e}")))?
        .into_body()
        .read_to_string()
        .map_err(|e| McmodError::Http(format!("{e}")))?;
    Ok(body)
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>> {
    let response = ureq::get(url)
        .header("User-Agent", "mcmod-cli")
        .call()
        .map_err(|e| McmodError::Http(format!("{e}")))?;

    let mut bytes = Vec::new();
    response
        .into_body()
        .as_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| McmodError::Http(format!("{e}")))?;
    Ok(bytes)
}
