use crate::config::McmodConfig;
use crate::error::{McmodError, Result};
use crate::gradle;
use crate::template::{self, render};
use crate::util::{derive_class_name, package_to_path, write_file};
use clap::ValueEnum;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

/// Features that can be added to an existing project.
#[derive(Clone, Debug, ValueEnum)]
pub enum Feature {
    Fabric,
    Neoforge,
    Ci,
    Kotlin,
    Publishing,
    Testing,
}

/// Dispatch an `add` subcommand.
pub fn run(feature: &Feature, dir: &Path) -> Result<()> {
    match feature {
        Feature::Fabric => run_add_fabric(dir),
        Feature::Neoforge => run_add_neoforge(dir),
        Feature::Ci => run_add_ci(dir),
        Feature::Kotlin => run_add_kotlin(dir),
        Feature::Publishing => run_add_publishing(dir),
        Feature::Testing => run_add_testing(dir),
    }
}

fn run_add_fabric(dir: &Path) -> Result<()> {
    println!("{}", "\n  mcmod add fabric\n".bold().cyan());
    let mut config = McmodConfig::load(dir)?;

    if config.loaders.fabric {
        return Err(McmodError::AlreadyEnabled("fabric".to_string()));
    }

    let vars = build_vars_from_config(&config);

    add_fabric_files(dir, &vars, &config.mod_info.language)?;

    // Update settings.gradle
    gradle::add_include_to_settings(dir, "fabric")?;

    // Update gradle.properties enabled_platforms
    gradle::add_platform_to_gradle_properties(dir, "fabric")?;

    // Update config
    config.loaders.fabric = true;
    config.save(dir)?;

    println!("{}", "  Fabric module added successfully!".bold().green());
    Ok(())
}

fn run_add_neoforge(dir: &Path) -> Result<()> {
    println!("{}", "\n  mcmod add neoforge\n".bold().cyan());
    let mut config = McmodConfig::load(dir)?;

    if config.loaders.neoforge {
        return Err(McmodError::AlreadyEnabled("neoforge".to_string()));
    }

    let vars = build_vars_from_config(&config);

    add_neoforge_files(dir, &vars, &config.mod_info.language)?;

    // Update settings.gradle
    gradle::add_include_to_settings(dir, "neoforge")?;

    // Update gradle.properties enabled_platforms
    gradle::add_platform_to_gradle_properties(dir, "neoforge")?;

    // Update config
    config.loaders.neoforge = true;
    config.save(dir)?;

    println!("{}", "  NeoForge module added successfully!".bold().green());
    Ok(())
}

fn run_add_ci(dir: &Path) -> Result<()> {
    println!("{}", "\n  mcmod add ci\n".bold().cyan());
    let mut config = McmodConfig::load(dir)?;

    if config.features.ci {
        return Err(McmodError::AlreadyEnabled("ci".to_string()));
    }

    let vars = build_vars_from_config(&config);

    add_ci_files(dir, &vars)?;

    // Update config
    config.features.ci = true;
    config.save(dir)?;

    println!("{}", "  CI workflow added successfully!".bold().green());
    Ok(())
}

fn run_add_kotlin(dir: &Path) -> Result<()> {
    println!("{}", "\n  mcmod add kotlin\n".bold().cyan());
    let mut config = McmodConfig::load(dir)?;

    if config.mod_info.language == "kotlin" {
        return Err(McmodError::AlreadyEnabled("kotlin".to_string()));
    }

    let vars = build_vars_from_config(&config);
    let package_path = package_to_path(&config.mod_info.package);
    let class_name = derive_class_name(&config.mod_info.mod_id);

    // Migrate common module
    migrate_to_kotlin(
        dir,
        "common",
        &package_path,
        &class_name,
        template::TMPL_COMMON_MOD_KT,
        &vars,
        None, // no mixin in common
    )?;

    // Migrate fabric module if present
    if config.loaders.fabric {
        migrate_to_kotlin(
            dir,
            "fabric",
            &format!("{package_path}/fabric"),
            &format!("{class_name}Fabric"),
            template::TMPL_FABRIC_MOD_KT,
            &vars,
            Some((&package_path, &config.mod_info.mod_name)),
        )?;
    }

    // Migrate neoforge module if present
    if config.loaders.neoforge {
        migrate_to_kotlin(
            dir,
            "neoforge",
            &format!("{package_path}/neoforge"),
            &format!("{class_name}NeoForge"),
            template::TMPL_NEOFORGE_MOD_KT,
            &vars,
            None,
        )?;
    }

    // Update gradle.properties
    gradle::set_gradle_property(dir, "mod_language", "kotlin")?;
    gradle::set_gradle_property(dir, "kotlin_version", "2.1.0")?;

    // Update config
    config.mod_info.language = "kotlin".to_string();
    config.save(dir)?;

    println!("{}", "  Kotlin migration completed successfully!".bold().green());
    Ok(())
}

fn run_add_publishing(dir: &Path) -> Result<()> {
    println!("{}", "\n  mcmod add publishing\n".bold().cyan());
    let mut config = McmodConfig::load(dir)?;

    if config.features.publishing {
        return Err(McmodError::AlreadyEnabled("publishing".to_string()));
    }

    let modrinth_id: String = dialoguer::Input::new()
        .with_prompt("  Modrinth project slug")
        .default(config.mod_info.mod_id.clone())
        .interact_text()
        .map_err(|e| McmodError::Other(e.to_string()))?;

    let cf_input: String = dialoguer::Input::new()
        .with_prompt("  CurseForge project ID (leave blank to skip)")
        .default(String::new())
        .interact_text()
        .map_err(|e| McmodError::Other(e.to_string()))?;
    let curseforge_id = if cf_input.is_empty() {
        None
    } else {
        Some(cf_input)
    };

    let mut vars = build_vars_from_config(&config);
    vars.insert("modrinth_id".to_string(), modrinth_id.clone());
    if let Some(ref id) = curseforge_id {
        vars.insert("curseforge_id".to_string(), id.clone());
    }

    add_publishing_files(
        dir,
        &vars,
        config.loaders.fabric,
        config.loaders.neoforge,
        curseforge_id.is_some(),
    )?;

    // Add version_type to gradle.properties if missing
    gradle::set_gradle_property(dir, "version_type", "release")?;

    // Update config
    config.features.publishing = true;
    config.publishing = Some(crate::config::Publishing {
        modrinth_id,
        curseforge_id,
    });
    config.save(dir)?;

    println!("{}", "  Publishing support added successfully!".bold().green());
    Ok(())
}

fn run_add_testing(dir: &Path) -> Result<()> {
    println!("{}", "\n  mcmod add testing\n".bold().cyan());
    let mut config = McmodConfig::load(dir)?;

    if config.features.testing {
        return Err(McmodError::AlreadyEnabled("testing".to_string()));
    }

    let vars = build_vars_from_config(&config);

    add_testing_files(
        dir,
        &vars,
        &config.mod_info.language,
        config.loaders.fabric,
        config.loaders.neoforge,
    )?;

    // Set testing_enabled in gradle.properties
    gradle::set_gradle_property(dir, "testing_enabled", "true")?;

    // Update config
    config.features.testing = true;
    config.save(dir)?;

    println!("{}", "  Testing support added successfully!".bold().green());
    Ok(())
}

/// Create testing files (used by both init and add).
pub fn add_testing_files(
    dir: &Path,
    vars: &HashMap<String, String>,
    language: &str,
    has_fabric: bool,
    has_neoforge: bool,
) -> Result<()> {
    let package_path = vars.get("package_path").unwrap();
    let class_name = vars.get("class_name").unwrap();

    let (test_tmpl, ext, source_dir) = if language == "kotlin" {
        (template::TMPL_COMMON_TEST_KT, "kt", "kotlin")
    } else {
        (template::TMPL_COMMON_TEST_JAVA, "java", "java")
    };

    // Unit test in common/src/test/
    write_file(
        &dir.join(format!(
            "common/src/test/{source_dir}/{package_path}/{class_name}Test.{ext}"
        )),
        &render(test_tmpl, vars)?,
    )?;
    println!("{}", "  Created common/ unit test".green());

    // Fabric GameTest
    if has_fabric {
        let fabric_tmpl = if language == "kotlin" {
            template::TMPL_FABRIC_GAMETEST_KT
        } else {
            template::TMPL_FABRIC_GAMETEST_JAVA
        };
        write_file(
            &dir.join(format!(
                "fabric/src/main/{source_dir}/{package_path}/fabric/{class_name}GameTest.{ext}"
            )),
            &render(fabric_tmpl, vars)?,
        )?;

        // Patch fabric.mod.json with gametest entrypoint
        let package = vars.get("package").unwrap();
        add_gametest_entrypoint(dir, package, class_name, language)?;
        println!("{}", "  Created fabric/ GameTest".green());
    }

    // NeoForge GameTest
    if has_neoforge {
        let neoforge_tmpl = if language == "kotlin" {
            template::TMPL_NEOFORGE_GAMETEST_KT
        } else {
            template::TMPL_NEOFORGE_GAMETEST_JAVA
        };
        write_file(
            &dir.join(format!(
                "neoforge/src/main/{source_dir}/{package_path}/neoforge/{class_name}GameTest.{ext}"
            )),
            &render(neoforge_tmpl, vars)?,
        )?;
        println!("{}", "  Created neoforge/ GameTest".green());
    }

    Ok(())
}

/// Patch fabric.mod.json to add the `fabric-gametest` entrypoint.
fn add_gametest_entrypoint(
    dir: &Path,
    package: &str,
    class_name: &str,
    _language: &str,
) -> Result<()> {
    let path = dir.join("fabric/src/main/resources/fabric.mod.json");
    let content = std::fs::read_to_string(&path)?;

    // Parse as JSON, add the entrypoint, and re-serialize
    let mut json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(entrypoints) = json.get_mut("entrypoints").and_then(|e| e.as_object_mut()) {
        let gametest_class = format!("{package}.fabric.{class_name}GameTest");
        entrypoints.insert(
            "fabric-gametest".to_string(),
            serde_json::json!([gametest_class]),
        );
    }

    // Write back with pretty formatting
    let formatted = serde_json::to_string_pretty(&json)?;
    std::fs::write(&path, formatted + "\n")?;
    Ok(())
}

/// Create publishing files (used by both init and add).
pub fn add_publishing_files(
    dir: &Path,
    vars: &HashMap<String, String>,
    has_fabric: bool,
    has_neoforge: bool,
    has_curseforge: bool,
) -> Result<()> {
    // Render and strip conditional blocks from release.yml
    let rendered = render(template::TMPL_CI_RELEASE_YML, vars)?;
    let stripped = template::strip_conditional_blocks(
        &rendered,
        &[
            ("fabric", has_fabric),
            ("neoforge", has_neoforge),
            ("curseforge", has_curseforge),
        ],
    );
    write_file(&dir.join(".github/workflows/release.yml"), &stripped)?;

    // Starter changelog
    write_file(
        &dir.join("changelogs/v1.0.0.md"),
        "Initial release.\n",
    )?;

    // Starter MODPAGE.md
    let mod_name = vars.get("mod_name").map(|s| s.as_str()).unwrap_or("My Mod");
    let description = vars
        .get("description")
        .map(|s| s.as_str())
        .unwrap_or("A Minecraft mod");
    write_file(
        &dir.join("MODPAGE.md"),
        &format!("# {mod_name}\n\n{description}\n"),
    )?;

    Ok(())
}

/// Migrate a module's Java source to Kotlin.
fn migrate_to_kotlin(
    dir: &Path,
    module: &str,
    source_package_path: &str,
    source_class_name: &str,
    kt_template: &str,
    vars: &HashMap<String, String>,
    mixin_info: Option<(&str, &str)>, // (package_path, mod_name) — only for fabric
) -> Result<()> {
    // Delete Java source file
    let java_path = dir.join(format!(
        "{module}/src/main/java/{source_package_path}/{source_class_name}.java"
    ));
    if java_path.exists() {
        std::fs::remove_file(&java_path)?;
        // Try to clean up empty java directories (but keep mixin package-info.java)
        cleanup_empty_dirs(&dir.join(format!("{module}/src/main/java/{source_package_path}")))?;
    }

    // Create Kotlin source file
    let kt_path = dir.join(format!(
        "{module}/src/main/kotlin/{source_package_path}/{source_class_name}.kt"
    ));
    write_file(&kt_path, &render(kt_template, vars)?)?;

    // If this is fabric, ensure mixin package-info.java stays in java tree
    if let Some((pkg_path, _)) = mixin_info {
        let mixin_java_path = dir.join(format!(
            "{module}/src/main/java/{pkg_path}/mixin/package-info.java"
        ));
        if !mixin_java_path.exists() {
            write_file(
                &mixin_java_path,
                &render(template::TMPL_FABRIC_MIXIN_PACKAGE_INFO, vars)?,
            )?;
        }
    }

    println!(
        "{}",
        format!("  Migrated {module}/ to Kotlin").green()
    );
    Ok(())
}

/// Remove a directory and its parents if they are empty.
fn cleanup_empty_dirs(path: &Path) -> Result<()> {
    let mut current = path.to_path_buf();
    while current.exists() {
        if std::fs::read_dir(&current)?.next().is_none() {
            std::fs::remove_dir(&current)?;
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    Ok(())
}

/// Create fabric module files (used by both init and add).
pub fn add_fabric_files(
    dir: &Path,
    vars: &HashMap<String, String>,
    language: &str,
) -> Result<()> {
    let package_path = vars.get("package_path").unwrap();
    let class_name = vars.get("class_name").unwrap();
    let mod_id = vars.get("mod_id").unwrap();

    // fabric/build.gradle.kts
    write_file(
        &dir.join("fabric/build.gradle.kts"),
        template::SC_FABRIC_BUILD_GRADLE,
    )?;

    // fabric/gradle.properties
    write_file(
        &dir.join("fabric/gradle.properties"),
        template::TMPL_FABRIC_GRADLE_PROPS,
    )?;

    // Source file
    let (tmpl, ext, source_dir) = if language == "kotlin" {
        (template::TMPL_FABRIC_MOD_KT, "kt", "kotlin")
    } else {
        (template::TMPL_FABRIC_MOD_JAVA, "java", "java")
    };
    write_file(
        &dir.join(format!(
            "fabric/src/main/{source_dir}/{package_path}/fabric/{class_name}Fabric.{ext}"
        )),
        &render(tmpl, vars)?,
    )?;

    // fabric.mod.json
    write_file(
        &dir.join("fabric/src/main/resources/fabric.mod.json"),
        &render(template::SC_FABRIC_MOD_JSON, vars)?,
    )?;

    // mixins.json
    write_file(
        &dir.join(format!(
            "fabric/src/main/resources/{mod_id}.mixins.json"
        )),
        &render(template::TMPL_FABRIC_MIXINS_JSON, vars)?,
    )?;

    // mixin package-info.java (always in java source tree, even for kotlin projects)
    write_file(
        &dir.join(format!(
            "fabric/src/main/java/{package_path}/mixin/package-info.java"
        )),
        &render(template::TMPL_FABRIC_MIXIN_PACKAGE_INFO, vars)?,
    )?;

    Ok(())
}

/// Create neoforge module files (used by both init and add).
pub fn add_neoforge_files(
    dir: &Path,
    vars: &HashMap<String, String>,
    language: &str,
) -> Result<()> {
    let package_path = vars.get("package_path").unwrap();
    let class_name = vars.get("class_name").unwrap();

    // neoforge/build.gradle.kts
    write_file(
        &dir.join("neoforge/build.gradle.kts"),
        template::SC_NEOFORGE_BUILD_GRADLE,
    )?;

    // neoforge/gradle.properties
    write_file(
        &dir.join("neoforge/gradle.properties"),
        template::TMPL_NEOFORGE_GRADLE_PROPS,
    )?;

    // Source file
    let (tmpl, ext, source_dir) = if language == "kotlin" {
        (template::TMPL_NEOFORGE_MOD_KT, "kt", "kotlin")
    } else {
        (template::TMPL_NEOFORGE_MOD_JAVA, "java", "java")
    };
    write_file(
        &dir.join(format!(
            "neoforge/src/main/{source_dir}/{package_path}/neoforge/{class_name}NeoForge.{ext}"
        )),
        &render(tmpl, vars)?,
    )?;

    // neoforge.mods.toml
    write_file(
        &dir.join("neoforge/src/main/resources/META-INF/neoforge.mods.toml"),
        &render(template::SC_NEOFORGE_MODS_TOML, vars)?,
    )?;

    Ok(())
}

/// Create CI files (used by both init and add).
pub fn add_ci_files(dir: &Path, vars: &HashMap<String, String>) -> Result<()> {
    write_file(
        &dir.join(".github/workflows/build.yml"),
        &render(template::TMPL_CI_BUILD_YML, vars)?,
    )?;
    Ok(())
}

/// Build template variables from an existing config.
fn build_vars_from_config(config: &McmodConfig) -> HashMap<String, String> {
    template::build_common_vars(config)
}
