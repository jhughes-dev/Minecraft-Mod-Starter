use std::collections::HashMap;

// Text templates (include_str!)
pub const TMPL_BUILD_GRADLE_ROOT: &str = include_str!("../templates/build.gradle.root");
pub const TMPL_SETTINGS_GRADLE: &str = include_str!("../templates/settings.gradle");
pub const TMPL_GRADLE_PROPERTIES: &str = include_str!("../templates/gradle.properties");
pub const TMPL_GITIGNORE: &str = include_str!("../templates/gitignore");
pub const TMPL_LICENSE: &str = include_str!("../templates/LICENSE");

pub const TMPL_COMMON_BUILD_GRADLE: &str = include_str!("../templates/common/build.gradle");
pub const TMPL_COMMON_MOD_JAVA: &str = include_str!("../templates/common/CommonMod.java");
pub const TMPL_COMMON_MOD_KT: &str = include_str!("../templates/common/CommonMod.kt");

pub const TMPL_FABRIC_BUILD_GRADLE: &str = include_str!("../templates/fabric/build.gradle");
pub const TMPL_FABRIC_GRADLE_PROPS: &str = include_str!("../templates/fabric/gradle.properties");
pub const TMPL_FABRIC_MOD_JAVA: &str = include_str!("../templates/fabric/FabricMod.java");
pub const TMPL_FABRIC_MOD_KT: &str = include_str!("../templates/fabric/FabricMod.kt");
pub const TMPL_FABRIC_MOD_JSON: &str = include_str!("../templates/fabric/fabric.mod.json");
pub const TMPL_FABRIC_MIXINS_JSON: &str = include_str!("../templates/fabric/mixins.json");
pub const TMPL_FABRIC_MIXIN_PACKAGE_INFO: &str =
    include_str!("../templates/fabric/mixin_package_info.java");

pub const TMPL_NEOFORGE_BUILD_GRADLE: &str = include_str!("../templates/neoforge/build.gradle");
pub const TMPL_NEOFORGE_GRADLE_PROPS: &str =
    include_str!("../templates/neoforge/gradle.properties");
pub const TMPL_NEOFORGE_MOD_JAVA: &str = include_str!("../templates/neoforge/NeoForgeMod.java");
pub const TMPL_NEOFORGE_MOD_KT: &str = include_str!("../templates/neoforge/NeoForgeMod.kt");
pub const TMPL_NEOFORGE_MODS_TOML: &str = include_str!("../templates/neoforge/neoforge.mods.toml");

pub const TMPL_CI_BUILD_YML: &str = include_str!("../templates/ci/build.yml");
pub const TMPL_CI_RELEASE_YML: &str = include_str!("../templates/ci/release.yml");

// Binary templates (include_bytes!)
pub const GRADLE_WRAPPER_JAR: &[u8] =
    include_bytes!("../templates/gradle-wrapper/gradle-wrapper.jar");
pub const GRADLE_WRAPPER_PROPS: &str =
    include_str!("../templates/gradle-wrapper/gradle-wrapper.properties");
pub const GRADLEW: &[u8] = include_bytes!("../templates/gradle-wrapper/gradlew");
pub const GRADLEW_BAT: &[u8] = include_bytes!("../templates/gradle-wrapper/gradlew.bat");

/// Render a template by replacing all `{{placeholder}}` occurrences with values from the map.
pub fn render(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Build the standard set of template variables from config values.
pub fn build_vars(
    mod_id: &str,
    mod_name: &str,
    package: &str,
    class_name: &str,
    author: &str,
    description: &str,
    language: &str,
    minecraft_version: &str,
    fabric_loader_version: &str,
    fabric_api_version: &str,
    neoforge_version: &str,
    enabled_platforms: &str,
    modrinth_id: Option<&str>,
    curseforge_id: Option<&str>,
) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("mod_id".to_string(), mod_id.to_string());
    vars.insert("mod_name".to_string(), mod_name.to_string());
    vars.insert("package".to_string(), package.to_string());
    vars.insert(
        "package_path".to_string(),
        crate::util::package_to_path(package),
    );
    vars.insert("class_name".to_string(), class_name.to_string());
    vars.insert("author".to_string(), author.to_string());
    vars.insert("description".to_string(), description.to_string());
    vars.insert("language".to_string(), language.to_string());
    vars.insert(
        "minecraft_version".to_string(),
        minecraft_version.to_string(),
    );
    vars.insert(
        "fabric_loader_version".to_string(),
        fabric_loader_version.to_string(),
    );
    vars.insert(
        "fabric_api_version".to_string(),
        fabric_api_version.to_string(),
    );
    vars.insert("neoforge_version".to_string(), neoforge_version.to_string());
    vars.insert(
        "neoforge_major".to_string(),
        crate::util::neoforge_major(neoforge_version),
    );
    vars.insert(
        "year".to_string(),
        chrono_year(),
    );
    vars.insert(
        "enabled_platforms".to_string(),
        enabled_platforms.to_string(),
    );
    if let Some(id) = modrinth_id {
        vars.insert("modrinth_id".to_string(), id.to_string());
    }
    if let Some(id) = curseforge_id {
        vars.insert("curseforge_id".to_string(), id.to_string());
    }
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
    // Use a simple approach: read from system time
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Approximate year calculation (good enough for copyright)
    let year = 1970 + secs / 31_557_600; // average seconds per year
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
            render("{{greeting}}, {{name}}!", &vars),
            "Hello, World!"
        );
    }

    #[test]
    fn test_render_no_placeholders() {
        let vars = HashMap::new();
        assert_eq!(render("no placeholders here", &vars), "no placeholders here");
    }

    #[test]
    fn test_render_multiple_occurrences() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "A".to_string());
        assert_eq!(render("{{x}} and {{x}}", &vars), "A and A");
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
