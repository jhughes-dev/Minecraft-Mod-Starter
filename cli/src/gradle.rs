use crate::error::Result;
use std::path::Path;

/// Add a loader to existing mc() calls in settings.gradle.kts.
///
/// Looks for lines matching `mc("X.Y.Z", ...)` and adds the loader argument
/// if not already present. For example, adding "neoforge" to
/// `mc("1.21.1", "fabric")` produces `mc("1.21.1", "fabric", "neoforge")`.
pub fn add_loader_to_settings_kts(dir: &Path, loader: &str) -> Result<()> {
    let path = dir.join("settings.gradle.kts");
    let content = std::fs::read_to_string(&path)?;
    let loader_arg = format!("\"{}\"", loader);

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed.starts_with("mc(\"") && trimmed.ends_with(')') {
            // Check if loader already present
            if line.contains(&loader_arg) {
                continue;
            }
            // Insert the loader before the closing paren
            if let Some(pos) = line.rfind(')') {
                line.insert_str(pos, &format!(", {loader_arg}"));
            }
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
    fn test_add_loader_to_settings_kts() {
        let dir = temp_dir("add_loader");
        fs::write(
            dir.join("settings.gradle.kts"),
            "        mc(\"1.21.1\", \"fabric\")\n        mc(\"1.21.7\", \"fabric\")\n",
        )
        .unwrap();

        add_loader_to_settings_kts(&dir, "neoforge").unwrap();
        let result = fs::read_to_string(dir.join("settings.gradle.kts")).unwrap();
        assert!(result.contains("mc(\"1.21.1\", \"fabric\", \"neoforge\")"));
        assert!(result.contains("mc(\"1.21.7\", \"fabric\", \"neoforge\")"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_loader_to_settings_kts_idempotent() {
        let dir = temp_dir("add_loader_idem");
        fs::write(
            dir.join("settings.gradle.kts"),
            "        mc(\"1.21.1\", \"fabric\", \"neoforge\")\n",
        )
        .unwrap();

        add_loader_to_settings_kts(&dir, "neoforge").unwrap();
        let result = fs::read_to_string(dir.join("settings.gradle.kts")).unwrap();
        assert_eq!(result.matches("\"neoforge\"").count(), 1);

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
}
