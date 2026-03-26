#![allow(dead_code)]

use crate::error::McmodError;
use crate::util::http_get;

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

/// Fetch the first stable version from a Fabric Meta API endpoint.
fn fetch_stable_from_fabric_meta(endpoint: &str, error_msg: &str) -> Result<String, McmodError> {
    let body = http_get(endpoint)?;
    let versions: Vec<serde_json::Value> = serde_json::from_str(&body)?;

    for v in &versions {
        if v.get("stable").and_then(|s| s.as_bool()) == Some(true) {
            if let Some(version) = v.get("version").and_then(|v| v.as_str()) {
                return Ok(version.to_string());
            }
        }
    }
    Err(McmodError::Other(error_msg.to_string()))
}

/// Fetch latest stable Minecraft version from Fabric Meta API.
pub fn fetch_minecraft_version() -> Result<String, McmodError> {
    fetch_stable_from_fabric_meta(
        "https://meta.fabricmc.net/v2/versions/game",
        "No stable Minecraft version found",
    )
}

/// Fetch latest stable Fabric Loader version from Fabric Meta API.
pub fn fetch_fabric_loader_version() -> Result<String, McmodError> {
    fetch_stable_from_fabric_meta(
        "https://meta.fabricmc.net/v2/versions/loader",
        "No stable Fabric Loader version found",
    )
}

/// Fetch latest Fabric API version for the given Minecraft version from Maven metadata.
pub fn fetch_fabric_api_version(mc_version: &str) -> Result<String, McmodError> {
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
pub fn fetch_neoforge_version(mc_version: &str) -> Result<String, McmodError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_maven_versions_basic() {
        let xml = r#"<?xml version="1.0"?>
<metadata>
  <versioning>
    <versions>
      <version>0.100.0+1.21.4</version>
      <version>0.101.0+1.21.4</version>
      <version>0.102.0+1.21.5</version>
    </versions>
  </versioning>
</metadata>"#;
        let versions = parse_maven_versions(xml);
        assert_eq!(
            versions,
            vec![
                "0.100.0+1.21.4",
                "0.101.0+1.21.4",
                "0.102.0+1.21.5",
            ]
        );
    }

    #[test]
    fn test_parse_maven_versions_empty() {
        let xml = "<metadata><versioning></versioning></metadata>";
        assert!(parse_maven_versions(xml).is_empty());
    }

    #[test]
    fn test_parse_maven_versions_ignores_non_version_lines() {
        let xml = r#"<metadata>
  <groupId>net.fabricmc</groupId>
  <artifactId>fabric-api</artifactId>
  <version>1.0.0</version>
</metadata>"#;
        let versions = parse_maven_versions(xml);
        assert_eq!(versions, vec!["1.0.0"]);
    }
}
