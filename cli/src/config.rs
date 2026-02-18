use crate::error::{McmodError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "mcmod.toml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McmodConfig {
    pub mod_info: ModInfo,
    pub loaders: Loaders,
    pub features: Features,
    pub versions: Versions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub mod_id: String,
    pub mod_name: String,
    pub package: String,
    pub author: String,
    pub description: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Loaders {
    pub fabric: bool,
    pub neoforge: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Features {
    pub ci: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Versions {
    pub minecraft: String,
    pub fabric_loader: String,
    pub fabric_api: String,
    pub neoforge: String,
}

impl McmodConfig {
    pub fn new(
        mod_id: String,
        mod_name: String,
        package: String,
        author: String,
        description: String,
        language: String,
        fabric: bool,
        neoforge: bool,
        ci: bool,
        versions: Versions,
    ) -> Self {
        Self {
            mod_info: ModInfo {
                mod_id,
                mod_name,
                package,
                author,
                description,
                language,
            },
            loaders: Loaders { fabric, neoforge },
            features: Features { ci },
            versions,
        }
    }

    /// Returns the list of enabled platform names (e.g. ["fabric", "neoforge"])
    pub fn enabled_platforms(&self) -> Vec<&str> {
        let mut platforms = Vec::new();
        if self.loaders.fabric {
            platforms.push("fabric");
        }
        if self.loaders.neoforge {
            platforms.push("neoforge");
        }
        platforms
    }

    /// Load config from mcmod.toml in the given directory.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(CONFIG_FILE);
        if !path.exists() {
            return Err(McmodError::ConfigNotFound);
        }
        let content = std::fs::read_to_string(&path)?;
        let config: McmodConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to mcmod.toml in the given directory.
    pub fn save(&self, dir: &Path) -> Result<()> {
        let path = dir.join(CONFIG_FILE);
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Returns the path to mcmod.toml for the given directory.
    #[allow(dead_code)]
    pub fn config_path(dir: &Path) -> PathBuf {
        dir.join(CONFIG_FILE)
    }
}

impl Default for Versions {
    fn default() -> Self {
        Self {
            minecraft: "1.21.4".to_string(),
            fabric_loader: "0.16.9".to_string(),
            fabric_api: "0.111.0+1.21.4".to_string(),
            neoforge: "21.4.156".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_roundtrip() {
        let config = McmodConfig::new(
            "mymod".to_string(),
            "My Mod".to_string(),
            "com.example.mymod".to_string(),
            "TestAuthor".to_string(),
            "A test mod".to_string(),
            "java".to_string(),
            true,
            true,
            false,
            Versions::default(),
        );

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: McmodConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.mod_info.mod_id, "mymod");
        assert_eq!(deserialized.mod_info.mod_name, "My Mod");
        assert_eq!(deserialized.loaders.fabric, true);
        assert_eq!(deserialized.loaders.neoforge, true);
        assert_eq!(deserialized.features.ci, false);
    }

    #[test]
    fn test_enabled_platforms() {
        let config = McmodConfig::new(
            "mymod".to_string(),
            "My Mod".to_string(),
            "com.example.mymod".to_string(),
            "Author".to_string(),
            "Desc".to_string(),
            "java".to_string(),
            true,
            false,
            false,
            Versions::default(),
        );
        assert_eq!(config.enabled_platforms(), vec!["fabric"]);

        let config2 = McmodConfig::new(
            "mymod".to_string(),
            "My Mod".to_string(),
            "com.example.mymod".to_string(),
            "Author".to_string(),
            "Desc".to_string(),
            "java".to_string(),
            true,
            true,
            false,
            Versions::default(),
        );
        assert_eq!(config2.enabled_platforms(), vec!["fabric", "neoforge"]);
    }
}
