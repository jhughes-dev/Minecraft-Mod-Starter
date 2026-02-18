use crate::error::Result;
use std::path::Path;

/// Add an `include("<module>")` line to settings.gradle.
/// Inserts it after the last existing `include(...)` line but before `rootProject.name`.
pub fn add_include_to_settings(dir: &Path, module: &str) -> Result<()> {
    let path = dir.join("settings.gradle");
    let content = std::fs::read_to_string(&path)?;
    let include_line = format!("include(\"{}\")", module);

    // Don't add if already present
    if content.contains(&include_line) {
        return Ok(());
    }

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    // Find the last include(...) line
    let mut last_include_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("include(") {
            last_include_idx = Some(i);
        }
    }

    if let Some(idx) = last_include_idx {
        lines.insert(idx + 1, include_line);
    } else {
        // No include lines found; insert before rootProject.name
        let rp_idx = lines
            .iter()
            .position(|l| l.trim().starts_with("rootProject.name"))
            .unwrap_or(lines.len());
        lines.insert(rp_idx, include_line);
    }

    let result = lines.join("\n");
    // Preserve trailing newline if original had one
    let result = if content.ends_with('\n') && !result.ends_with('\n') {
        result + "\n"
    } else {
        result
    };
    std::fs::write(&path, result)?;
    Ok(())
}

/// Update the `enabled_platforms` value in gradle.properties to include a new platform.
pub fn add_platform_to_gradle_properties(dir: &Path, platform: &str) -> Result<()> {
    let path = dir.join("gradle.properties");
    let content = std::fs::read_to_string(&path)?;

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    for line in &mut lines {
        if line.starts_with("enabled_platforms=") {
            let current = line.trim_start_matches("enabled_platforms=");
            let platforms: Vec<&str> = current.split(',').map(|s| s.trim()).collect();
            if !platforms.contains(&platform) {
                let mut new_platforms: Vec<&str> = platforms;
                new_platforms.push(platform);
                *line = format!("enabled_platforms={}", new_platforms.join(","));
            }
            break;
        }
    }

    let result = lines.join("\n");
    let result = if content.ends_with('\n') && !result.ends_with('\n') {
        result + "\n"
    } else {
        result
    };
    std::fs::write(&path, result)?;
    Ok(())
}

/// Set or add a property in gradle.properties.
pub fn set_gradle_property(dir: &Path, key: &str, value: &str) -> Result<()> {
    let path = dir.join("gradle.properties");
    let content = std::fs::read_to_string(&path)?;

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let prefix = format!("{key}=");
    let commented_prefix = format!("# {key}=");
    let new_line = format!("{key}={value}");

    let mut found = false;
    for line in &mut lines {
        if line.starts_with(&prefix) || line.starts_with(&commented_prefix) {
            *line = new_line.clone();
            found = true;
            break;
        }
    }

    if !found {
        // Add at the end, before any trailing blank lines
        lines.push(new_line);
    }

    let result = lines.join("\n");
    let result = if content.ends_with('\n') && !result.ends_with('\n') {
        result + "\n"
    } else {
        result
    };
    std::fs::write(&path, result)?;
    Ok(())
}
