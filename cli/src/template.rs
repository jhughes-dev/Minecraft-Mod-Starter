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
    vars
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
}
