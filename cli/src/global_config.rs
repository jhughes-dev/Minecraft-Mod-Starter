use crate::error::{McmodError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = "config.toml";

#[derive(Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub defaults: GlobalDefaults,
    #[serde(default)]
    pub options: ClientOptions,
    #[serde(default)]
    pub gamerules: GameRuleDefaults,
}

#[derive(Serialize, Deserialize, Default)]
pub struct GlobalDefaults {
    pub author: Option<String>,
    pub language: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientOptions {
    pub fullscreen: Option<bool>,
    pub pause_on_lost_focus: Option<bool>,
    pub auto_jump: Option<bool>,
    pub reduced_debug_info: Option<bool>,
    pub gamma: Option<f64>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            fullscreen: Some(true),
            pause_on_lost_focus: Some(false),
            auto_jump: Some(false),
            reduced_debug_info: Some(false),
            gamma: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameRuleDefaults {
    pub do_daylight_cycle: Option<bool>,
    pub do_weather_cycle: Option<bool>,
    pub time_of_day: Option<String>,
}

impl Default for GameRuleDefaults {
    fn default() -> Self {
        Self {
            do_daylight_cycle: Some(false),
            do_weather_cycle: Some(false),
            time_of_day: Some("noon".to_string()),
        }
    }
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
            "options.fullscreen" => self.options.fullscreen.map(|v| v.to_string()),
            "options.pause_on_lost_focus" => self.options.pause_on_lost_focus.map(|v| v.to_string()),
            "options.auto_jump" => self.options.auto_jump.map(|v| v.to_string()),
            "options.reduced_debug_info" => self.options.reduced_debug_info.map(|v| v.to_string()),
            "options.gamma" => self.options.gamma.map(|v| v.to_string()),
            "gamerules.do_daylight_cycle" => self.gamerules.do_daylight_cycle.map(|v| v.to_string()),
            "gamerules.do_weather_cycle" => self.gamerules.do_weather_cycle.map(|v| v.to_string()),
            "gamerules.time_of_day" => self.gamerules.time_of_day.clone(),
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
            "options.fullscreen" => {
                self.options.fullscreen = Some(parse_bool(value)?);
            }
            "options.pause_on_lost_focus" => {
                self.options.pause_on_lost_focus = Some(parse_bool(value)?);
            }
            "options.auto_jump" => {
                self.options.auto_jump = Some(parse_bool(value)?);
            }
            "options.reduced_debug_info" => {
                self.options.reduced_debug_info = Some(parse_bool(value)?);
            }
            "options.gamma" => {
                let v: f64 = value.parse().map_err(|_| {
                    McmodError::Other(format!("Invalid gamma value '{value}': must be a number"))
                })?;
                self.options.gamma = Some(v);
            }
            "gamerules.do_daylight_cycle" => {
                self.gamerules.do_daylight_cycle = Some(parse_bool(value)?);
            }
            "gamerules.do_weather_cycle" => {
                self.gamerules.do_weather_cycle = Some(parse_bool(value)?);
            }
            "gamerules.time_of_day" => {
                validate_time_of_day(value)?;
                self.gamerules.time_of_day = Some(value.to_lowercase());
            }
            _ => {
                return Err(McmodError::Other(format!(
                    "Unknown config key '{key}'. Run 'mcmod config list' to see valid keys."
                )));
            }
        }
        self.save()
    }

    /// List all config key-value pairs, grouped by section.
    /// Returns (section_name, key, display_value) tuples.
    pub fn list(&self) -> Vec<(&'static str, String, String)> {
        let mut entries = Vec::new();

        let display = |v: &Option<String>| {
            v.clone().unwrap_or_else(|| "(not set)".to_string())
        };
        let display_bool = |v: &Option<bool>| match v {
            Some(b) => b.to_string(),
            None => "(not set)".to_string(),
        };
        let display_f64 = |v: &Option<f64>| match v {
            Some(f) => f.to_string(),
            None => "(not set)".to_string(),
        };

        // Defaults
        entries.push(("Defaults", "author".to_string(), display(&self.defaults.author)));
        entries.push(("Defaults", "language".to_string(), display(&self.defaults.language)));

        // Client Options
        entries.push(("Client Options", "fullscreen".to_string(), display_bool(&self.options.fullscreen)));
        entries.push(("Client Options", "pauseOnLostFocus".to_string(), display_bool(&self.options.pause_on_lost_focus)));
        entries.push(("Client Options", "autoJump".to_string(), display_bool(&self.options.auto_jump)));
        entries.push(("Client Options", "reducedDebugInfo".to_string(), display_bool(&self.options.reduced_debug_info)));
        entries.push(("Client Options", "gamma".to_string(), display_f64(&self.options.gamma)));

        // Game Rules
        entries.push(("Game Rules", "doDaylightCycle".to_string(), display_bool(&self.gamerules.do_daylight_cycle)));
        entries.push(("Game Rules", "doWeatherCycle".to_string(), display_bool(&self.gamerules.do_weather_cycle)));
        entries.push(("Game Rules", "timeOfDay".to_string(), display(&self.gamerules.time_of_day)));

        entries
    }

    /// Render options.txt content from the current config.
    pub fn render_options_txt(&self) -> String {
        let mut lines = Vec::new();
        lines.push("lang:en_us".to_string());

        if let Some(v) = self.options.fullscreen {
            lines.push(format!("fullscreen:{v}"));
        }
        if let Some(v) = self.options.pause_on_lost_focus {
            lines.push(format!("pauseOnLostFocus:{v}"));
        }
        if let Some(v) = self.options.auto_jump {
            lines.push(format!("autoJump:{v}"));
        }
        if let Some(v) = self.options.reduced_debug_info {
            lines.push(format!("reducedDebugInfo:{v}"));
        }
        if let Some(v) = self.options.gamma {
            lines.push(format!("gamma:{v}"));
        }

        lines.push(String::new()); // trailing newline
        lines.join("\n")
    }
}

/// Normalize short key names to their dotted form.
/// Accepts both camelCase and snake_case short forms.
fn normalize_key(key: &str) -> String {
    match key {
        // Defaults
        "author" => "defaults.author".to_string(),
        "language" => "defaults.language".to_string(),

        // Client Options — camelCase
        "fullscreen" => "options.fullscreen".to_string(),
        "pauseOnLostFocus" | "pause_on_lost_focus" => "options.pause_on_lost_focus".to_string(),
        "autoJump" | "auto_jump" => "options.auto_jump".to_string(),
        "reducedDebugInfo" | "reduced_debug_info" => "options.reduced_debug_info".to_string(),
        "gamma" => "options.gamma".to_string(),

        // Game Rules — camelCase and snake_case
        "doDaylightCycle" | "do_daylight_cycle" => "gamerules.do_daylight_cycle".to_string(),
        "doWeatherCycle" | "do_weather_cycle" => "gamerules.do_weather_cycle".to_string(),
        "timeOfDay" | "time_of_day" => "gamerules.time_of_day".to_string(),

        other => other.to_string(),
    }
}

/// Parse a boolean value accepting true/false/yes/no/1/0.
fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err(McmodError::Other(format!(
            "Invalid boolean '{value}': must be true/false/yes/no/1/0"
        ))),
    }
}

/// Validate a time-of-day value. Accepts named times or raw tick numbers.
fn validate_time_of_day(value: &str) -> Result<()> {
    let lower = value.to_lowercase();
    match lower.as_str() {
        "noon" | "day" | "midnight" | "night" | "sunrise" | "sunset" => Ok(()),
        _ => {
            // Accept raw tick number
            value.parse::<u32>().map_err(|_| {
                McmodError::Other(format!(
                    "Invalid time '{value}': must be noon/day/midnight/night/sunrise/sunset or a tick number"
                ))
            })?;
            Ok(())
        }
    }
}

/// Convert a time-of-day name to its Minecraft tick value for mcfunction commands.
pub fn time_to_tick(time: &str) -> &str {
    match time.to_lowercase().as_str() {
        "noon" | "day" => "day",
        "midnight" | "night" => "midnight",
        "sunrise" => "23000",
        "sunset" => "12000",
        _ => time, // raw tick number passed through
    }
}

/// Copies options.txt generated from config into the given path.
/// No-op if the destination already exists.
pub fn copy_options_to(dest: &Path, config: &GlobalConfig) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }
    let content = config.render_options_txt();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest, content)?;
    Ok(())
}

/// Maps a Minecraft version string to the correct data pack pack_format number.
/// Returns (major, minor) where minor is 0 for pre-1.21.9 versions.
fn mc_version_to_pack_format(mc_version: &str) -> (u32, u32) {
    match mc_version {
        "1.21" | "1.21.1" => (48, 0),
        "1.21.2" | "1.21.3" => (57, 0),
        "1.21.4" => (61, 0),
        "1.21.5" => (71, 0),
        "1.21.6" => (80, 0),
        "1.21.7" | "1.21.8" => (81, 0),
        "1.21.9" | "1.21.10" => (88, 0),
        "1.21.11" => (94, 1),
        _ => {
            // For unknown versions, try to guess based on the minor version number.
            // Parse the third component if present.
            let parts: Vec<&str> = mc_version.splitn(3, '.').collect();
            if parts.len() == 3 {
                if let Ok(minor) = parts[2].parse::<u32>() {
                    if minor >= 11 {
                        return (94, 1); // latest known
                    } else if minor >= 9 {
                        return (88, 0);
                    }
                }
            }
            // Default fallback to 1.21.4's format
            (61, 0)
        }
    }
}

/// Returns true if the MC version uses the new min_format/max_format pack.mcmeta
/// scheme (introduced in 1.21.9).
fn uses_new_pack_format(mc_version: &str) -> bool {
    let (major, _) = mc_version_to_pack_format(mc_version);
    major >= 88
}

/// Renders the pack.mcmeta JSON for the given Minecraft version.
fn render_pack_mcmeta(mc_version: &str) -> String {
    let (major, minor) = mc_version_to_pack_format(mc_version);
    if uses_new_pack_format(mc_version) {
        // 1.21.9+ uses min_format/max_format alongside pack_format
        if minor > 0 {
            format!(
                "{{\n  \"pack\": {{\n    \"pack_format\": [{major}, {minor}],\n    \"min_format\": [{major}, 0],\n    \"max_format\": [{major}, {minor}],\n    \"description\": \"Dev defaults (generated by mcmod)\"\n  }}\n}}\n"
            )
        } else {
            format!(
                "{{\n  \"pack\": {{\n    \"pack_format\": {major},\n    \"min_format\": {major},\n    \"max_format\": {major},\n    \"description\": \"Dev defaults (generated by mcmod)\"\n  }}\n}}\n"
            )
        }
    } else {
        // Pre-1.21.9 uses only pack_format
        format!(
            "{{\n  \"pack\": {{\n    \"pack_format\": {major},\n    \"description\": \"Dev defaults (generated by mcmod)\"\n  }}\n}}\n"
        )
    }
}

/// Writes a dev-defaults data pack into the project's run/world directory.
/// The data pack sets game rules on world load via a mcfunction.
/// `mc_version` determines the correct pack_format for pack.mcmeta.
pub fn write_dev_datapack(project_dir: &Path, config: &GlobalConfig, mc_version: &str) -> Result<()> {
    let pack_dir = project_dir.join("run/world/datapacks/dev-defaults");

    // pack.mcmeta — version-aware format
    crate::util::write_file(
        &pack_dir.join("pack.mcmeta"),
        &render_pack_mcmeta(mc_version),
    )?;

    // load function tag — runs dev:init on world load
    crate::util::write_file(
        &pack_dir.join("data/minecraft/tags/function/load.json"),
        "{\n  \"values\": [\n    \"dev:init\"\n  ]\n}\n",
    )?;

    // init.mcfunction — generated from gamerule config
    let mut commands = Vec::new();

    if let Some(v) = config.gamerules.do_daylight_cycle {
        commands.push(format!("gamerule doDaylightCycle {v}"));
    }
    if let Some(v) = config.gamerules.do_weather_cycle {
        commands.push(format!("gamerule doWeatherCycle {v}"));
    }
    if let Some(ref time) = config.gamerules.time_of_day {
        commands.push(format!("time set {}", time_to_tick(time)));
    }

    if !commands.is_empty() {
        commands.push(String::new()); // trailing newline
    }

    crate::util::write_file(
        &pack_dir.join("data/dev/function/init.mcfunction"),
        &commands.join("\n"),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_key_defaults() {
        assert_eq!(normalize_key("author"), "defaults.author");
        assert_eq!(normalize_key("language"), "defaults.language");
    }

    #[test]
    fn test_normalize_key_options() {
        assert_eq!(normalize_key("fullscreen"), "options.fullscreen");
        assert_eq!(normalize_key("pauseOnLostFocus"), "options.pause_on_lost_focus");
        assert_eq!(normalize_key("pause_on_lost_focus"), "options.pause_on_lost_focus");
        assert_eq!(normalize_key("autoJump"), "options.auto_jump");
        assert_eq!(normalize_key("auto_jump"), "options.auto_jump");
        assert_eq!(normalize_key("gamma"), "options.gamma");
    }

    #[test]
    fn test_normalize_key_gamerules() {
        assert_eq!(normalize_key("doDaylightCycle"), "gamerules.do_daylight_cycle");
        assert_eq!(normalize_key("do_daylight_cycle"), "gamerules.do_daylight_cycle");
        assert_eq!(normalize_key("doWeatherCycle"), "gamerules.do_weather_cycle");
        assert_eq!(normalize_key("timeOfDay"), "gamerules.time_of_day");
        assert_eq!(normalize_key("time_of_day"), "gamerules.time_of_day");
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("true").unwrap());
        assert!(parse_bool("yes").unwrap());
        assert!(parse_bool("1").unwrap());
        assert!(!parse_bool("false").unwrap());
        assert!(!parse_bool("no").unwrap());
        assert!(!parse_bool("0").unwrap());
        assert!(parse_bool("maybe").is_err());
    }

    #[test]
    fn test_validate_time_of_day() {
        assert!(validate_time_of_day("noon").is_ok());
        assert!(validate_time_of_day("day").is_ok());
        assert!(validate_time_of_day("midnight").is_ok());
        assert!(validate_time_of_day("night").is_ok());
        assert!(validate_time_of_day("sunrise").is_ok());
        assert!(validate_time_of_day("sunset").is_ok());
        assert!(validate_time_of_day("6000").is_ok());
        assert!(validate_time_of_day("banana").is_err());
    }

    #[test]
    fn test_time_to_tick() {
        assert_eq!(time_to_tick("noon"), "day");
        assert_eq!(time_to_tick("day"), "day");
        assert_eq!(time_to_tick("midnight"), "midnight");
        assert_eq!(time_to_tick("night"), "midnight");
        assert_eq!(time_to_tick("sunrise"), "23000");
        assert_eq!(time_to_tick("sunset"), "12000");
        assert_eq!(time_to_tick("6000"), "6000");
    }

    #[test]
    fn test_render_options_txt_defaults() {
        let config = GlobalConfig::default();
        let txt = config.render_options_txt();
        assert!(txt.contains("lang:en_us"));
        assert!(txt.contains("fullscreen:true"));
        assert!(txt.contains("pauseOnLostFocus:false"));
        assert!(txt.contains("autoJump:false"));
        assert!(txt.contains("reducedDebugInfo:false"));
        // gamma not set by default, should not appear
        assert!(!txt.contains("gamma:"));
    }

    #[test]
    fn test_render_options_txt_custom() {
        let mut config = GlobalConfig::default();
        config.options.fullscreen = Some(false);
        config.options.gamma = Some(1.5);
        let txt = config.render_options_txt();
        assert!(txt.contains("fullscreen:false"));
        assert!(txt.contains("gamma:1.5"));
    }

    #[test]
    fn test_default_config_deserializes_from_empty() {
        let config: GlobalConfig = toml::from_str("").unwrap();
        // Should get defaults for options and gamerules
        assert_eq!(config.options.fullscreen, Some(true));
        assert_eq!(config.options.pause_on_lost_focus, Some(false));
        assert_eq!(config.gamerules.do_daylight_cycle, Some(false));
        assert_eq!(config.gamerules.time_of_day, Some("noon".to_string()));
    }

    #[test]
    fn test_backward_compatible_config() {
        // Old config files only had [defaults] section
        let toml_str = r#"
[defaults]
author = "TestAuthor"
language = "java"
"#;
        let config: GlobalConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.defaults.author, Some("TestAuthor".to_string()));
        // Should still get defaults for new sections
        assert_eq!(config.options.auto_jump, Some(false));
        assert_eq!(config.gamerules.do_weather_cycle, Some(false));
    }

    #[test]
    fn test_list_returns_all_sections() {
        let config = GlobalConfig::default();
        let entries = config.list();
        let sections: Vec<&str> = entries.iter().map(|(s, _, _)| *s).collect();
        assert!(sections.contains(&"Defaults"));
        assert!(sections.contains(&"Client Options"));
        assert!(sections.contains(&"Game Rules"));
        assert_eq!(entries.len(), 10);
    }

    #[test]
    fn test_mc_version_to_pack_format() {
        assert_eq!(mc_version_to_pack_format("1.21"), (48, 0));
        assert_eq!(mc_version_to_pack_format("1.21.1"), (48, 0));
        assert_eq!(mc_version_to_pack_format("1.21.2"), (57, 0));
        assert_eq!(mc_version_to_pack_format("1.21.3"), (57, 0));
        assert_eq!(mc_version_to_pack_format("1.21.4"), (61, 0));
        assert_eq!(mc_version_to_pack_format("1.21.5"), (71, 0));
        assert_eq!(mc_version_to_pack_format("1.21.6"), (80, 0));
        assert_eq!(mc_version_to_pack_format("1.21.7"), (81, 0));
        assert_eq!(mc_version_to_pack_format("1.21.8"), (81, 0));
        assert_eq!(mc_version_to_pack_format("1.21.9"), (88, 0));
        assert_eq!(mc_version_to_pack_format("1.21.10"), (88, 0));
        assert_eq!(mc_version_to_pack_format("1.21.11"), (94, 1));
    }

    #[test]
    fn test_uses_new_pack_format() {
        assert!(!uses_new_pack_format("1.21.4"));
        assert!(!uses_new_pack_format("1.21.8"));
        assert!(uses_new_pack_format("1.21.9"));
        assert!(uses_new_pack_format("1.21.11"));
    }

    #[test]
    fn test_render_pack_mcmeta_old_format() {
        let mcmeta = render_pack_mcmeta("1.21.4");
        assert!(mcmeta.contains("\"pack_format\": 61"));
        assert!(!mcmeta.contains("min_format"));
        assert!(!mcmeta.contains("max_format"));
    }

    #[test]
    fn test_render_pack_mcmeta_new_format_no_minor() {
        let mcmeta = render_pack_mcmeta("1.21.9");
        assert!(mcmeta.contains("\"pack_format\": 88"));
        assert!(mcmeta.contains("\"min_format\": 88"));
        assert!(mcmeta.contains("\"max_format\": 88"));
    }

    #[test]
    fn test_render_pack_mcmeta_new_format_with_minor() {
        let mcmeta = render_pack_mcmeta("1.21.11");
        assert!(mcmeta.contains("\"pack_format\": [94, 1]"));
        assert!(mcmeta.contains("\"min_format\": [94, 0]"));
        assert!(mcmeta.contains("\"max_format\": [94, 1]"));
    }

    #[test]
    fn test_unknown_version_fallback() {
        // Unknown future version with high minor should use latest known
        assert_eq!(mc_version_to_pack_format("1.21.15"), (94, 1));
        // Completely unknown version falls back to 1.21.4's format
        assert_eq!(mc_version_to_pack_format("1.22"), (61, 0));
    }
}
