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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("mcmod_gradle_{name}_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_add_include_to_settings_after_existing() {
        let dir = temp_dir("include_after");
        fs::write(
            dir.join("settings.gradle"),
            "include(\"common\")\nrootProject.name = \"mymod\"\n",
        )
        .unwrap();

        add_include_to_settings(&dir, "fabric").unwrap();
        let result = fs::read_to_string(dir.join("settings.gradle")).unwrap();
        assert!(result.contains("include(\"fabric\")"));
        let common_pos = result.find("include(\"common\")").unwrap();
        let fabric_pos = result.find("include(\"fabric\")").unwrap();
        assert!(fabric_pos > common_pos);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_include_to_settings_idempotent() {
        let dir = temp_dir("include_idem");
        fs::write(
            dir.join("settings.gradle"),
            "include(\"common\")\ninclude(\"fabric\")\nrootProject.name = \"mymod\"\n",
        )
        .unwrap();

        add_include_to_settings(&dir, "fabric").unwrap();
        let result = fs::read_to_string(dir.join("settings.gradle")).unwrap();
        assert_eq!(result.matches("include(\"fabric\")").count(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_set_gradle_property_existing() {
        let dir = temp_dir("prop_existing");
        fs::write(
            dir.join("gradle.properties"),
            "mod_id=test\nmod_version=1.0.0\n",
        )
        .unwrap();

        set_gradle_property(&dir, "mod_version", "2.0.0").unwrap();
        let result = fs::read_to_string(dir.join("gradle.properties")).unwrap();
        assert!(result.contains("mod_version=2.0.0"));
        assert!(!result.contains("mod_version=1.0.0"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_set_gradle_property_commented() {
        let dir = temp_dir("prop_commented");
        fs::write(
            dir.join("gradle.properties"),
            "mod_id=test\n# kotlin_version=1.9.0\n",
        )
        .unwrap();

        set_gradle_property(&dir, "kotlin_version", "2.1.0").unwrap();
        let result = fs::read_to_string(dir.join("gradle.properties")).unwrap();
        assert!(result.contains("kotlin_version=2.1.0"));
        assert!(!result.contains("# kotlin_version"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_set_gradle_property_new() {
        let dir = temp_dir("prop_new");
        fs::write(dir.join("gradle.properties"), "mod_id=test\n").unwrap();

        set_gradle_property(&dir, "new_key", "new_value").unwrap();
        let result = fs::read_to_string(dir.join("gradle.properties")).unwrap();
        assert!(result.contains("new_key=new_value"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_platform_to_gradle_properties() {
        let dir = temp_dir("add_platform");
        fs::write(
            dir.join("gradle.properties"),
            "enabled_platforms=fabric\n",
        )
        .unwrap();

        add_platform_to_gradle_properties(&dir, "neoforge").unwrap();
        let result = fs::read_to_string(dir.join("gradle.properties")).unwrap();
        assert!(result.contains("enabled_platforms=fabric,neoforge"));

        let _ = fs::remove_dir_all(&dir);
    }
}
