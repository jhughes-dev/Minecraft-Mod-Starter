use crate::config::Versions;
use crate::error::McmodError;
use crate::util::http_get;
use colored::Colorize;

/// Fetch the latest versions from online APIs, falling back to defaults on failure.
/// If `offline` is true, returns defaults immediately without fetching.
pub fn fetch_versions(offline: bool) -> Versions {
    if offline {
        println!("{}", "  Using offline defaults for versions".yellow());
        return Versions::default();
    }

    println!("{}", "  Fetching latest versions...".cyan());

    let minecraft = fetch_minecraft_version().unwrap_or_else(|e| {
        eprintln!(
            "{}",
            format!("  Warning: Could not fetch Minecraft version: {e}").yellow()
        );
        Versions::default().minecraft
    });

    let fabric_loader = fetch_fabric_loader_version().unwrap_or_else(|e| {
        eprintln!(
            "{}",
            format!("  Warning: Could not fetch Fabric Loader version: {e}").yellow()
        );
        Versions::default().fabric_loader
    });

    let fabric_api = fetch_fabric_api_version(&minecraft).unwrap_or_else(|e| {
        eprintln!(
            "{}",
            format!("  Warning: Could not fetch Fabric API version: {e}").yellow()
        );
        Versions::default().fabric_api
    });

    let neoforge = fetch_neoforge_version(&minecraft).unwrap_or_else(|e| {
        eprintln!(
            "{}",
            format!("  Warning: Could not fetch NeoForge version: {e}").yellow()
        );
        Versions::default().neoforge
    });

    println!(
        "{}",
        format!(
            "  Minecraft: {minecraft}, Fabric Loader: {fabric_loader}, Fabric API: {fabric_api}, NeoForge: {neoforge}"
        )
        .green()
    );

    Versions {
        minecraft,
        fabric_loader,
        fabric_api,
        neoforge,
    }
}

/// Parse `<version>` tags from Maven metadata XML, returning all version strings.
fn parse_maven_versions(xml: &str) -> Vec<String> {
    xml.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
                Some(trimmed[9..trimmed.len() - 10].to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Fetch latest stable Minecraft version from Fabric Meta API.
fn fetch_minecraft_version() -> Result<String, McmodError> {
    let body = http_get("https://meta.fabricmc.net/v2/versions/game")?;
    let versions: Vec<serde_json::Value> = serde_json::from_str(&body)?;

    for v in &versions {
        if v.get("stable").and_then(|s| s.as_bool()) == Some(true) {
            if let Some(version) = v.get("version").and_then(|v| v.as_str()) {
                return Ok(version.to_string());
            }
        }
    }
    Err(McmodError::Other(
        "No stable Minecraft version found".to_string(),
    ))
}

/// Fetch latest stable Fabric Loader version from Fabric Meta API.
fn fetch_fabric_loader_version() -> Result<String, McmodError> {
    let body = http_get("https://meta.fabricmc.net/v2/versions/loader")?;
    let versions: Vec<serde_json::Value> = serde_json::from_str(&body)?;

    for v in &versions {
        if v.get("stable").and_then(|s| s.as_bool()) == Some(true) {
            if let Some(version) = v.get("version").and_then(|v| v.as_str()) {
                return Ok(version.to_string());
            }
        }
    }
    Err(McmodError::Other(
        "No stable Fabric Loader version found".to_string(),
    ))
}

/// Fetch latest Fabric API version for the given Minecraft version from Maven metadata.
fn fetch_fabric_api_version(mc_version: &str) -> Result<String, McmodError> {
    let url = "https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/maven-metadata.xml";
    let body = http_get(url)?;
    let suffix = format!("+{mc_version}");

    let matching: Vec<String> = parse_maven_versions(&body)
        .into_iter()
        .filter(|v| v.ends_with(&suffix))
        .collect();

    matching
        .last()
        .cloned()
        .ok_or_else(|| McmodError::Other(format!("No Fabric API version found for {mc_version}")))
}

/// Fetch latest NeoForge version for the given Minecraft version from Maven metadata.
fn fetch_neoforge_version(mc_version: &str) -> Result<String, McmodError> {
    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
    let body = http_get(url)?;

    // NeoForge versions follow the pattern {mc_major}.{mc_minor}.xxx
    // For MC 1.21.4, NeoForge versions are 21.4.xxx
    let parts: Vec<&str> = mc_version.splitn(3, '.').collect();
    let prefix = if parts.len() >= 3 {
        format!("{}.{}.", parts[1], parts[2])
    } else if parts.len() == 2 {
        format!("{}.", parts[1])
    } else {
        return Err(McmodError::Other(format!(
            "Cannot parse Minecraft version: {mc_version}"
        )));
    };

    let matching: Vec<String> = parse_maven_versions(&body)
        .into_iter()
        .filter(|v| v.starts_with(&prefix))
        .collect();

    matching
        .last()
        .cloned()
        .ok_or_else(|| {
            McmodError::Other(format!("No NeoForge version found for {mc_version}"))
        })
}
