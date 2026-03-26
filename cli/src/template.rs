use crate::config::{McmodConfig, VersionTarget};
use crate::error::{McmodError, Result};
use std::collections::HashMap;

// --- Legacy templates (still used by source file generation & add command) ---
pub const TMPL_GITIGNORE: &str = include_str!("../templates/gitignore");
pub const TMPL_LICENSE: &str = include_str!("../templates/LICENSE");

pub const TMPL_COMMON_MOD_JAVA: &str = include_str!("../templates/common/CommonMod.java");
pub const TMPL_COMMON_MOD_KT: &str = include_str!("../templates/common/CommonMod.kt");

pub const TMPL_FABRIC_MOD_JAVA: &str = include_str!("../templates/fabric/FabricMod.java");
pub const TMPL_FABRIC_MOD_KT: &str = include_str!("../templates/fabric/FabricMod.kt");
pub const TMPL_FABRIC_MIXINS_JSON: &str = include_str!("../templates/fabric/mixins.json");
pub const TMPL_FABRIC_MIXIN_PACKAGE_INFO: &str =
    include_str!("../templates/fabric/mixin_package_info.java");

pub const TMPL_NEOFORGE_MOD_JAVA: &str = include_str!("../templates/neoforge/NeoForgeMod.java");
pub const TMPL_NEOFORGE_MOD_KT: &str = include_str!("../templates/neoforge/NeoForgeMod.kt");

pub const TMPL_COMMON_TEST_JAVA: &str = include_str!("../templates/common/ExampleModTest.java");
pub const TMPL_COMMON_TEST_KT: &str = include_str!("../templates/common/ExampleModTest.kt");
pub const TMPL_FABRIC_GAMETEST_JAVA: &str = include_str!("../templates/fabric/FabricGameTest.java");
pub const TMPL_FABRIC_GAMETEST_KT: &str = include_str!("../templates/fabric/FabricGameTest.kt");
pub const TMPL_NEOFORGE_GAMETEST_JAVA: &str =
    include_str!("../templates/neoforge/NeoForgeGameTest.java");
pub const TMPL_NEOFORGE_GAMETEST_KT: &str =
    include_str!("../templates/neoforge/NeoForgeGameTest.kt");

pub const TMPL_CI_BUILD_YML: &str = include_str!("../templates/ci/build.yml");
pub const TMPL_CI_RELEASE_YML: &str = include_str!("../templates/ci/release.yml");

// --- Stonecutter templates ---
pub const SC_SETTINGS_GRADLE: &str =
    include_str!("../templates/stonecutter/settings.gradle.kts");
pub const SC_STONECUTTER_GRADLE: &str =
    include_str!("../templates/stonecutter/stonecutter.gradle.kts");
pub const SC_BUILD_GRADLE: &str =
    include_str!("../templates/stonecutter/build.gradle.kts");
pub const SC_GRADLE_PROPERTIES: &str =
    include_str!("../templates/stonecutter/gradle.properties");
pub const SC_VERSION_GRADLE_PROPERTIES: &str =
    include_str!("../templates/stonecutter/version.gradle.properties");
pub const SC_FABRIC_BUILD_GRADLE: &str =
    include_str!("../templates/stonecutter/fabric/build.gradle.kts");
pub const SC_FABRIC_MOD_JSON: &str =
    include_str!("../templates/stonecutter/fabric/fabric.mod.json");
pub const SC_NEOFORGE_BUILD_GRADLE: &str =
    include_str!("../templates/stonecutter/neoforge/build.gradle.kts");
pub const SC_NEOFORGE_MODS_TOML: &str =
    include_str!("../templates/stonecutter/neoforge/neoforge.mods.toml");

// Fabric/NeoForge platform gradle.properties (unchanged — just sets loom.platform)
pub const TMPL_FABRIC_GRADLE_PROPS: &str = include_str!("../templates/fabric/gradle.properties");
pub const TMPL_NEOFORGE_GRADLE_PROPS: &str =
    include_str!("../templates/neoforge/gradle.properties");

// --- Binary templates (include_bytes!) ---
pub const GRADLE_WRAPPER_JAR: &[u8] =
    include_bytes!("../templates/gradle-wrapper/gradle-wrapper.jar");
pub const GRADLE_WRAPPER_PROPS: &str =
    include_str!("../templates/gradle-wrapper/gradle-wrapper.properties");
pub const GRADLEW: &[u8] = include_bytes!("../templates/gradle-wrapper/gradlew");
pub const GRADLEW_BAT: &[u8] = include_bytes!("../templates/gradle-wrapper/gradlew.bat");

/// Render a template by replacing all `{{placeholder}}` occurrences with values from the map.
/// Returns an error if any `{{placeholder}}` patterns remain after substitution.
pub fn render(template: &str, vars: &HashMap<String, String>) -> Result<String> {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }

    // Check for unreplaced placeholders (but not conditional block markers like {{#name}}/{{/name}}
    // and not GitHub Actions expressions like ${{ ... }})
    let mut pos = 0;
    while let Some(start) = result[pos..].find("{{") {
        let abs_start = pos + start;
        if let Some(end) = result[abs_start..].find("}}") {
            let inner = &result[abs_start + 2..abs_start + end];
            // Skip conditional block markers ({{#...}} and {{/...}})
            // Skip GitHub Actions expressions (${{...}})
            let is_gha = abs_start > 0 && result.as_bytes()[abs_start - 1] == b'$';
            if !inner.starts_with('#') && !inner.starts_with('/') && !is_gha {
                return Err(McmodError::Other(format!(
                    "Unreplaced template placeholder: {{{{{}}}}}",
                    inner
                )));
            }
            pos = abs_start + end + 2;
        } else {
            break;
        }
    }

    Ok(result)
}

/// Build the common template variables from an McmodConfig.
/// These are used for all templates rendered at init time.
pub fn build_common_vars(config: &McmodConfig) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("mod_id".to_string(), config.mod_info.mod_id.clone());
    vars.insert("mod_name".to_string(), config.mod_info.mod_name.clone());
    vars.insert("package".to_string(), config.mod_info.package.clone());
    vars.insert(
        "package_path".to_string(),
        crate::util::package_to_path(&config.mod_info.package),
    );
    vars.insert(
        "class_name".to_string(),
        crate::util::derive_class_name(&config.mod_info.mod_id),
    );
    vars.insert("author".to_string(), config.mod_info.author.clone());
    vars.insert(
        "description".to_string(),
        config.mod_info.description.clone(),
    );
    vars.insert("language".to_string(), config.mod_info.language.clone());
    vars.insert("year".to_string(), chrono_year());

    // Stonecutter-specific
    vars.insert(
        "stonecutter_versions".to_string(),
        config.stonecutter_versions(),
    );
    vars.insert(
        "active_version".to_string(),
        config.active_version().to_string(),
    );
    vars.insert(
        "architectury_plugin_version".to_string(),
        config.versions.architectury_plugin.clone(),
    );
    vars.insert(
        "architectury_loom_version".to_string(),
        config.versions.architectury_loom.clone(),
    );

    if let Some(ref pub_config) = config.publishing {
        vars.insert("modrinth_id".to_string(), pub_config.modrinth_id.clone());
        if let Some(ref id) = pub_config.curseforge_id {
            vars.insert("curseforge_id".to_string(), id.clone());
        }
    }
    vars
}

/// Build per-version template variables for a specific VersionTarget.
/// Used to render the per-version gradle.properties.
pub fn build_version_vars(target: &VersionTarget) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert(
        "fabric_loader_version".to_string(),
        target.fabric_loader.clone(),
    );
    vars.insert(
        "fabric_api_version".to_string(),
        target.fabric_api.clone(),
    );
    vars.insert("neoforge_version".to_string(), target.neoforge.clone());
    vars.insert("mc_dep_fabric".to_string(), target.mc_dep_fabric());
    vars.insert("mc_dep_neoforge".to_string(), target.mc_dep_neoforge());
    vars.insert("neoforge_dep".to_string(), target.neoforge_dep());
    vars
}

/// Strip conditional blocks from rendered template content.
///
/// Blocks are delimited by `{{#name}}...{{/name}}` markers (each on its own line).
/// If a condition is true the markers are removed but the content is kept.
/// If false the entire block (markers + content) is removed.
pub fn strip_conditional_blocks(content: &str, conditions: &[(&str, bool)]) -> String {
    let mut result = content.to_string();
    for &(name, enabled) in conditions {
        let open = format!("{{{{#{}}}}}", name);
        let close = format!("{{{{/{}}}}}", name);

        if enabled {
            // Keep content, remove markers (and their surrounding newline)
            result = result.replace(&format!("{open}\n"), "");
            result = result.replace(&format!("{close}\n"), "");
            // Handle markers without trailing newline (e.g. at end of file)
            result = result.replace(&open, "");
            result = result.replace(&close, "");
        } else {
            // Remove entire block including markers
            let mut out = String::new();
            let mut skip = false;
            for line in result.lines() {
                let trimmed = line.trim();
                if trimmed == open {
                    skip = true;
                    continue;
                }
                if trimmed == close {
                    skip = false;
                    continue;
                }
                if !skip {
                    out.push_str(line);
                    out.push('\n');
                }
            }
            result = out;
        }
    }
    // Collapse 3+ consecutive blank lines into 2
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }
    result
}

fn chrono_year() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Convert unix timestamp to year using proper Gregorian calendar math
    let days = (secs / 86400) as i64;
    // Days from Unix epoch (1970-01-01) — use the civil_from_days algorithm
    // Shift epoch from 1970-01-01 to 0000-03-01 for easier leap year handling
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    year.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        vars.insert("greeting".to_string(), "Hello".to_string());
        assert_eq!(
            render("{{greeting}}, {{name}}!", &vars).unwrap(),
            "Hello, World!"
        );
    }

    #[test]
    fn test_render_no_placeholders() {
        let vars = HashMap::new();
        assert_eq!(
            render("no placeholders here", &vars).unwrap(),
            "no placeholders here"
        );
    }

    #[test]
    fn test_render_multiple_occurrences() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "A".to_string());
        assert_eq!(render("{{x}} and {{x}}", &vars).unwrap(), "A and A");
    }

    #[test]
    fn test_render_unreplaced_placeholder_errors() {
        let vars = HashMap::new();
        let result = render("Hello {{name}}!", &vars);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_render_ignores_conditional_blocks() {
        let vars = HashMap::new();
        let result = render("{{#fabric}}\ncontent\n{{/fabric}}", &vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_ignores_github_actions_expressions() {
        let vars = HashMap::new();
        let result = render("run: echo ${{ github.event.inputs.version_type }}", &vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_strip_conditional_blocks_enabled() {
        let input = "before\n{{#fabric}}\nfabric content\n{{/fabric}}\nafter\n";
        let result = strip_conditional_blocks(input, &[("fabric", true)]);
        assert!(result.contains("fabric content"));
        assert!(!result.contains("{{#fabric}}"));
        assert!(!result.contains("{{/fabric}}"));
    }

    #[test]
    fn test_strip_conditional_blocks_disabled() {
        let input = "before\n{{#curseforge}}\ncurseforge content\n{{/curseforge}}\nafter\n";
        let result = strip_conditional_blocks(input, &[("curseforge", false)]);
        assert!(!result.contains("curseforge content"));
        assert!(result.contains("before"));
        assert!(result.contains("after"));
    }

    #[test]
    fn test_strip_conditional_blocks_nested() {
        let input = "{{#fabric}}\nouter\n{{#curseforge}}\ninner\n{{/curseforge}}\nrest\n{{/fabric}}\n";
        let result = strip_conditional_blocks(input, &[("fabric", true), ("curseforge", false)]);
        assert!(result.contains("outer"));
        assert!(!result.contains("inner"));
        assert!(result.contains("rest"));
    }
}
