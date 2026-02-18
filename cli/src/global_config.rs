use crate::error::{McmodError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = "config.toml";
const OPTIONS_FILENAME: &str = "options.txt";

/// Default content for a new options.txt (matches Minecraft defaults for common settings).
const DEFAULT_OPTIONS: &str = "lang:en_us\n";

#[derive(Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub defaults: GlobalDefaults,
}

#[derive(Serialize, Deserialize, Default)]
pub struct GlobalDefaults {
    pub author: Option<String>,
    pub language: Option<String>,
}

/// Returns the platform-specific global config directory for mcmod.
/// - Linux/macOS: $XDG_CONFIG_HOME/mcmod or ~/.config/mcmod
/// - Windows: %APPDATA%/mcmod
pub fn global_config_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Ok(PathBuf::from(appdata).join("mcmod"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg).join("mcmod"));
        }
    }

    // Fallback: ~/.config/mcmod
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| McmodError::Other("Could not determine home directory".to_string()))?;
    Ok(PathBuf::from(home).join(".config").join("mcmod"))
}

impl GlobalConfig {
    /// Load global config from config.toml. Returns Default if file is missing or corrupt.
    pub fn load() -> Result<Self> {
        let dir = global_config_dir()?;
        let path = dir.join(CONFIG_FILENAME);
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let config: GlobalConfig =
            toml::from_str(&content).unwrap_or_default();
        Ok(config)
    }

    /// Save global config to config.toml, creating the directory if needed.
    pub fn save(&self) -> Result<()> {
        let dir = global_config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(CONFIG_FILENAME);
        let content = toml::to_string_pretty(self)
            .map_err(|e| McmodError::TomlSerialize(e))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get a config value by key. Accepts short keys like "author" or dotted "defaults.author".
    pub fn get(&self, key: &str) -> Option<String> {
        let normalized = normalize_key(key);
        match normalized.as_str() {
            "defaults.author" => self.defaults.author.clone(),
            "defaults.language" => self.defaults.language.clone(),
            _ => None,
        }
    }

    /// Set a config value by key. Validates known keys and language values.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let normalized = normalize_key(key);
        match normalized.as_str() {
            "defaults.author" => {
                self.defaults.author = Some(value.to_string());
            }
            "defaults.language" => {
                let lower = value.to_lowercase();
                if lower != "java" && lower != "kotlin" {
                    return Err(McmodError::Other(format!(
                        "Invalid language '{value}': must be 'java' or 'kotlin'"
                    )));
                }
                self.defaults.language = Some(lower);
            }
            _ => {
                return Err(McmodError::Other(format!(
                    "Unknown config key '{key}'. Valid keys: author, language"
                )));
            }
        }
        self.save()
    }

    /// List all config key-value pairs.
    pub fn list(&self) -> Vec<(String, String)> {
        let mut entries = Vec::new();
        entries.push((
            "author".to_string(),
            self.defaults
                .author
                .clone()
                .unwrap_or_else(|| "(not set)".to_string()),
        ));
        entries.push((
            "language".to_string(),
            self.defaults
                .language
                .clone()
                .unwrap_or_else(|| "(not set)".to_string()),
        ));
        entries
    }
}

/// Normalize short key names to their dotted form.
fn normalize_key(key: &str) -> String {
    match key {
        "author" => "defaults.author".to_string(),
        "language" => "defaults.language".to_string(),
        other => other.to_string(),
    }
}

/// Ensures the global options.txt exists in the config directory.
/// Returns the path to the options.txt file.
pub fn ensure_global_options() -> Result<PathBuf> {
    let dir = global_config_dir()?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(OPTIONS_FILENAME);
    if !path.exists() {
        std::fs::write(&path, DEFAULT_OPTIONS)?;
    }
    Ok(path)
}

/// Links or copies options.txt from target_path to link_path (cross-platform).
/// Tries to create a symlink first; falls back to copying on Windows if
/// symlinks aren't available (Developer Mode disabled).
/// No-op if the link/file already exists.
/// Returns true if a symlink was created, false if a copy was used.
pub fn create_options_symlink(link_path: &Path, target_path: &Path) -> Result<bool> {
    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        return Ok(true);
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target_path, link_path)?;
        return Ok(true);
    }

    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_file(target_path, link_path) {
            Ok(()) => return Ok(true),
            Err(_) => {
                // Symlinks unavailable — fall back to copying the file
                std::fs::copy(target_path, link_path)?;
                return Ok(false);
            }
        }
    }
}
