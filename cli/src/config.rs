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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publishing: Option<Publishing>,
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
    #[serde(default)]
    pub publishing: bool,
    #[serde(default)]
    pub testing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Publishing {
    pub modrinth_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curseforge_id: Option<String>,
}

/// Version configuration for a Stonecutter multi-version project.
///
/// `targets` lists each MC version to build against. Each target holds
/// the per-version dependency versions and the upper bound of its
/// compatibility range. Shared toolchain versions live at the top level.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Versions {
    pub targets: Vec<VersionTarget>,
    pub architectury_plugin: String,
    pub architectury_loom: String,
}

/// A single Minecraft version target and its per-version dependencies.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionTarget {
    /// The MC version to compile against (e.g. "1.21.1").
    pub minecraft: String,
    /// Upper bound of the compatibility range (e.g. "1.21.6").
    pub max_minecraft: String,
    pub fabric_loader: String,
    pub fabric_api: String,
    pub neoforge: String,
}

impl VersionTarget {
    /// Fabric mod.json dependency range: `>=1.21.1 <=1.21.6`
    pub fn mc_dep_fabric(&self) -> String {
        if self.minecraft == self.max_minecraft {
            format!("~{}", self.minecraft)
        } else {
            format!(">={} <={}", self.minecraft, self.max_minecraft)
        }
    }

    /// NeoForge mods.toml dependency range: `[1.21.1,1.21.6]`
    pub fn mc_dep_neoforge(&self) -> String {
        if self.minecraft == self.max_minecraft {
            format!("[{},)", self.minecraft)
        } else {
            format!("[{},{}]", self.minecraft, self.max_minecraft)
        }
    }

    /// NeoForge loader dependency range: `[21.1,)` derived from neoforge version.
    pub fn neoforge_dep(&self) -> String {
        let major = crate::util::neoforge_major(&self.neoforge);
        format!("[{major},)")
    }
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
        testing: bool,
        publishing: Option<Publishing>,
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
            features: Features {
                ci,
                publishing: publishing.is_some(),
                testing,
            },
            versions,
            publishing,
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

    /// The first (lowest) target MC version — used as the Stonecutter active version.
    pub fn active_version(&self) -> &str {
        self.versions
            .targets
            .first()
            .map(|t| t.minecraft.as_str())
            .unwrap_or("1.21.4")
    }

    /// Stonecutter versions string: `"1.21.1", "1.21.7"`
    pub fn stonecutter_versions(&self) -> String {
        self.versions
            .targets
            .iter()
            .map(|t| format!("\"{}\"", t.minecraft))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for Versions {
    fn default() -> Self {
        Self {
            targets: vec![VersionTarget {
                minecraft: "1.21.4".to_string(),
                max_minecraft: "1.21.4".to_string(),
                fabric_loader: "0.18.5".to_string(),
                fabric_api: "0.119.4+1.21.4".to_string(),
                neoforge: "21.4.157".to_string(),
            }],
            architectury_plugin: "3.4.162".to_string(),
            architectury_loom: "1.13.469".to_string(),
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
            false,
            None,
            Versions::default(),
        );

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: McmodConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.mod_info.mod_id, "mymod");
        assert_eq!(deserialized.mod_info.mod_name, "My Mod");
        assert_eq!(deserialized.loaders.fabric, true);
        assert_eq!(deserialized.loaders.neoforge, true);
        assert_eq!(deserialized.features.ci, false);
        assert_eq!(deserialized.versions.targets.len(), 1);
        assert_eq!(deserialized.versions.targets[0].minecraft, "1.21.4");
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
            false,
            None,
            Versions::default(),
        );
        assert_eq!(config.enabled_platforms(), vec!["fabric"]);
    }

    #[test]
    fn test_version_target_deps() {
        let target = VersionTarget {
            minecraft: "1.21.1".to_string(),
            max_minecraft: "1.21.6".to_string(),
            fabric_loader: "0.18.5".to_string(),
            fabric_api: "0.116.9+1.21.1".to_string(),
            neoforge: "21.1.221".to_string(),
        };
        assert_eq!(target.mc_dep_fabric(), ">=1.21.1 <=1.21.6");
        assert_eq!(target.mc_dep_neoforge(), "[1.21.1,1.21.6]");
        assert_eq!(target.neoforge_dep(), "[21.1,)");
    }

    #[test]
    fn test_version_target_single_version() {
        let target = VersionTarget {
            minecraft: "1.21.4".to_string(),
            max_minecraft: "1.21.4".to_string(),
            fabric_loader: "0.18.5".to_string(),
            fabric_api: "0.119.4+1.21.4".to_string(),
            neoforge: "21.4.157".to_string(),
        };
        assert_eq!(target.mc_dep_fabric(), "~1.21.4");
        assert_eq!(target.mc_dep_neoforge(), "[1.21.4,)");
    }

    #[test]
    fn test_stonecutter_versions_string() {
        let versions = Versions {
            targets: vec![
                VersionTarget {
                    minecraft: "1.21.1".to_string(),
                    max_minecraft: "1.21.6".to_string(),
                    fabric_loader: "0.18.5".to_string(),
                    fabric_api: "0.116.9+1.21.1".to_string(),
                    neoforge: "21.1.221".to_string(),
                },
                VersionTarget {
                    minecraft: "1.21.7".to_string(),
                    max_minecraft: "1.21.11".to_string(),
                    fabric_loader: "0.18.5".to_string(),
                    fabric_api: "0.128.2+1.21.7".to_string(),
                    neoforge: "21.7.25-beta".to_string(),
                },
            ],
            architectury_plugin: "3.4.162".to_string(),
            architectury_loom: "1.13.469".to_string(),
        };
        let config = McmodConfig::new(
            "test".to_string(),
            "Test".to_string(),
            "com.test".to_string(),
            "Author".to_string(),
            "Desc".to_string(),
            "java".to_string(),
            true,
            true,
            false,
            false,
            None,
            versions,
        );
        assert_eq!(config.stonecutter_versions(), "\"1.21.1\", \"1.21.7\"");
        assert_eq!(config.active_version(), "1.21.1");
    }
}
