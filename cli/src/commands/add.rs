use crate::config::McmodConfig;
use crate::error::{McmodError, Result};
use crate::gradle;
use crate::template::{self, render};
use crate::util::{derive_class_name, package_to_path, write_file};
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

/// Dispatch an `add` subcommand.
pub fn run(feature: &str, dir: &Path) -> Result<()> {
    match feature {
        "fabric" => run_add_fabric(dir),
        "neoforge" => run_add_neoforge(dir),
        "ci" => run_add_ci(dir),
        "kotlin" => run_add_kotlin(dir),
        _ => Err(McmodError::Other(format!(
            "Unknown feature: {feature}. Valid features: fabric, neoforge, ci, kotlin"
        ))),
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
    write_file(&kt_path, &render(kt_template, vars))?;

    // If this is fabric, ensure mixin package-info.java stays in java tree
    if let Some((pkg_path, _)) = mixin_info {
        let mixin_java_path = dir.join(format!(
            "{module}/src/main/java/{pkg_path}/mixin/package-info.java"
        ));
        if !mixin_java_path.exists() {
            write_file(
                &mixin_java_path,
                &render(template::TMPL_FABRIC_MIXIN_PACKAGE_INFO, vars),
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

    // fabric/build.gradle
    write_file(
        &dir.join("fabric/build.gradle"),
        template::TMPL_FABRIC_BUILD_GRADLE,
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
        &render(tmpl, vars),
    )?;

    // fabric.mod.json
    write_file(
        &dir.join("fabric/src/main/resources/fabric.mod.json"),
        &render(template::TMPL_FABRIC_MOD_JSON, vars),
    )?;

    // mixins.json
    let mod_id = vars.get("mod_id").unwrap();
    write_file(
        &dir.join(format!(
            "fabric/src/main/resources/{mod_id}.mixins.json"
        )),
        &render(template::TMPL_FABRIC_MIXINS_JSON, vars),
    )?;

    // mixin package-info.java (always in java source tree, even for kotlin projects)
    write_file(
        &dir.join(format!(
            "fabric/src/main/java/{package_path}/mixin/package-info.java"
        )),
        &render(template::TMPL_FABRIC_MIXIN_PACKAGE_INFO, vars),
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

    // neoforge/build.gradle
    write_file(
        &dir.join("neoforge/build.gradle"),
        template::TMPL_NEOFORGE_BUILD_GRADLE,
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
        &render(tmpl, vars),
    )?;

    // neoforge.mods.toml
    write_file(
        &dir.join("neoforge/src/main/resources/META-INF/neoforge.mods.toml"),
        &render(template::TMPL_NEOFORGE_MODS_TOML, vars),
    )?;

    Ok(())
}

/// Create CI files (used by both init and add).
pub fn add_ci_files(dir: &Path, vars: &HashMap<String, String>) -> Result<()> {
    write_file(
        &dir.join(".github/workflows/build.yml"),
        &render(template::TMPL_CI_BUILD_YML, vars),
    )?;
    Ok(())
}

/// Build template variables from an existing config.
fn build_vars_from_config(config: &McmodConfig) -> HashMap<String, String> {
    let class_name = derive_class_name(&config.mod_info.mod_id);
    template::build_vars(
        &config.mod_info.mod_id,
        &config.mod_info.mod_name,
        &config.mod_info.package,
        &class_name,
        &config.mod_info.author,
        &config.mod_info.description,
        &config.mod_info.language,
        &config.versions.minecraft,
        &config.versions.fabric_loader,
        &config.versions.fabric_api,
        &config.versions.neoforge,
        &config.enabled_platforms().join(","),
    )
}
