use crate::commands::add;
use crate::config::McmodConfig;
use crate::error::Result;
use crate::template::{self, render};
use crate::util::{derive_class_name, write_binary, write_file};
use crate::versions::fetch_versions;
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct InitOptions {
    pub dir: PathBuf,
    pub mod_id: Option<String>,
    pub mod_name: Option<String>,
    pub package: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub loaders: Vec<String>,
    pub ci: Option<bool>,
    pub offline: bool,
}

pub fn run(opts: InitOptions) -> Result<()> {
    println!("{}", "\n  mcmod init\n".bold().cyan());

    let interactive = opts.mod_id.is_none();

    // Load global config for defaults (never blocks init)
    let global = crate::global_config::GlobalConfig::load().unwrap_or_default();

    // Gather inputs
    let mod_id = if let Some(id) = opts.mod_id {
        id
    } else {
        prompt_input("Mod ID", "mymod")?
    };
    crate::util::validate_mod_id(&mod_id)?;

    let mod_name = if let Some(name) = opts.mod_name {
        name
    } else {
        let default = default_mod_name(&mod_id);
        prompt_input("Mod Name", &default)?
    };

    let package = if let Some(pkg) = opts.package {
        pkg
    } else {
        let default = format!("com.example.{mod_id}");
        prompt_input("Package", &default)?
    };
    crate::util::validate_package(&package)?;

    let author = if let Some(a) = opts.author {
        a
    } else {
        let default_author = global.defaults.author.as_deref().unwrap_or("Your Name");
        prompt_input("Author", default_author)?
    };

    let description = if let Some(d) = opts.description {
        d
    } else {
        prompt_input("Description", "A Minecraft mod")?
    };

    let language = if let Some(l) = opts.language {
        l
    } else if interactive {
        let default_idx = match global.defaults.language.as_deref() {
            Some("kotlin") => 1,
            _ => 0,
        };
        prompt_select("Language", &["java", "kotlin"], default_idx)?
    } else {
        global.defaults.language.as_deref().unwrap_or("java").to_string()
    };

    let loaders = if !opts.loaders.is_empty() {
        opts.loaders
    } else if interactive {
        prompt_multiselect("Loaders", &["fabric", "neoforge"])?
    } else {
        vec!["fabric".to_string(), "neoforge".to_string()]
    };

    if loaders.is_empty() {
        return Err(crate::error::McmodError::Other(
            "At least one loader must be selected".to_string(),
        ));
    }

    let ci = if let Some(c) = opts.ci {
        c
    } else if interactive {
        prompt_confirm("Enable CI (GitHub Actions)?", true)?
    } else {
        true
    };

    let offline = opts.offline;

    // Fetch versions
    let versions = fetch_versions(offline);

    // Derive values
    let class_name = derive_class_name(&mod_id);
    let has_fabric = loaders.iter().any(|l| l == "fabric");
    let has_neoforge = loaders.iter().any(|l| l == "neoforge");

    let enabled_platforms = loaders.join(",");

    let vars = template::build_vars(
        &mod_id,
        &mod_name,
        &package,
        &class_name,
        &author,
        &description,
        &language,
        &versions.minecraft,
        &versions.fabric_loader,
        &versions.fabric_api,
        &versions.neoforge,
        &enabled_platforms,
    );

    // Create project directory
    let project_dir = &opts.dir;
    crate::util::ensure_dir(project_dir)?;

    println!("{}", format!("  Creating project in {}", project_dir.display()).cyan());

    // Write base files
    write_base_files(project_dir, &vars, &language)?;

    // Write common module
    write_common_module(project_dir, &vars, &language)?;

    // Write loader modules
    if has_fabric {
        add::add_fabric_files(project_dir, &vars, &language)?;
        crate::gradle::add_include_to_settings(project_dir, "fabric")?;
        println!("{}", "  Created fabric/ module".green());
    }
    if has_neoforge {
        add::add_neoforge_files(project_dir, &vars, &language)?;
        crate::gradle::add_include_to_settings(project_dir, "neoforge")?;
        println!("{}", "  Created neoforge/ module".green());
    }

    // Copy global options.txt template into run/ (shared by both loaders)
    match create_run_options(project_dir) {
        Ok(()) => println!("{}", "  Created run/options.txt".green()),
        Err(e) => eprintln!("  {}", format!("Warning: Could not create options.txt: {e}").yellow()),
    }

    // Write CI
    if ci {
        add::add_ci_files(project_dir, &vars)?;
        println!("{}", "  Created .github/workflows/build.yml".green());
    }

    // Write mcmod.toml
    let config = McmodConfig::new(
        mod_id.clone(),
        mod_name.clone(),
        package.clone(),
        author.clone(),
        description.clone(),
        language.clone(),
        has_fabric,
        has_neoforge,
        ci,
        versions,
    );
    config.save(project_dir)?;

    // Print success
    println!("\n{}", "  Project created successfully!".bold().green());
    println!();
    println!("  {}", format!("  Mod ID:      {mod_id}").white());
    println!("  {}", format!("  Mod Name:    {mod_name}").white());
    println!("  {}", format!("  Package:     {package}").white());
    println!("  {}", format!("  Language:    {language}").white());
    println!(
        "  {}",
        format!("  Loaders:     {}", loaders.join(", ")).white()
    );
    println!("  {}", format!("  CI:          {ci}").white());
    println!();
    println!(
        "  {}",
        "  Next steps:".bold()
    );
    println!("    cd {}", project_dir.display());
    println!("    ./gradlew build");
    println!();

    Ok(())
}

fn write_base_files(dir: &Path, vars: &HashMap<String, String>, language: &str) -> Result<()> {
    // build.gradle (root) — no placeholders, copy as-is
    write_file(
        &dir.join("build.gradle"),
        template::TMPL_BUILD_GRADLE_ROOT,
    )?;

    // settings.gradle — has {{mod_id}} placeholder
    write_file(
        &dir.join("settings.gradle"),
        &render(template::TMPL_SETTINGS_GRADLE, vars),
    )?;

    // gradle.properties
    let mut props = render(template::TMPL_GRADLE_PROPERTIES, vars);
    // Add language properties if kotlin
    if language == "kotlin" {
        props.push_str("\nmod_language=kotlin\nkotlin_version=2.1.0\n");
    }
    write_file(&dir.join("gradle.properties"), &props)?;

    // .gitignore
    write_file(&dir.join(".gitignore"), template::TMPL_GITIGNORE)?;

    // LICENSE
    write_file(&dir.join("LICENSE"), &render(template::TMPL_LICENSE, vars))?;

    // Gradle wrapper
    write_binary(
        &dir.join("gradle/wrapper/gradle-wrapper.jar"),
        template::GRADLE_WRAPPER_JAR,
    )?;
    write_file(
        &dir.join("gradle/wrapper/gradle-wrapper.properties"),
        template::GRADLE_WRAPPER_PROPS,
    )?;
    write_binary(&dir.join("gradlew"), template::GRADLEW)?;
    write_binary(&dir.join("gradlew.bat"), template::GRADLEW_BAT)?;

    // Set gradlew as executable (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(dir.join("gradlew"))?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(dir.join("gradlew"), perms)?;
    }

    println!("{}", "  Created base project files".green());
    Ok(())
}

fn write_common_module(
    dir: &Path,
    vars: &HashMap<String, String>,
    language: &str,
) -> Result<()> {
    let package_path = vars.get("package_path").unwrap();

    // common/build.gradle
    write_file(
        &dir.join("common/build.gradle"),
        template::TMPL_COMMON_BUILD_GRADLE,
    )?;

    // Source file
    let (template, ext, source_dir) = if language == "kotlin" {
        (template::TMPL_COMMON_MOD_KT, "kt", "kotlin")
    } else {
        (template::TMPL_COMMON_MOD_JAVA, "java", "java")
    };

    let class_name = vars.get("class_name").unwrap();
    let source_path = dir.join(format!(
        "common/src/main/{source_dir}/{package_path}/{class_name}.{ext}"
    ));
    write_file(&source_path, &render(template, vars))?;

    // assets/<mod_id>/icon.png.txt
    let mod_id = vars.get("mod_id").unwrap();
    write_file(
        &dir.join(format!(
            "common/src/main/resources/assets/{mod_id}/icon.png.txt"
        )),
        "Replace this file with your mod icon (icon.png)\n",
    )?;

    println!("{}", "  Created common/ module".green());
    Ok(())
}

// --- Prompt helpers ---

fn prompt_input(prompt: &str, default: &str) -> Result<String> {
    let result = dialoguer::Input::<String>::new()
        .with_prompt(format!("  {prompt}"))
        .default(default.to_string())
        .interact_text()
        .map_err(|e| crate::error::McmodError::Other(e.to_string()))?;
    Ok(result)
}

fn prompt_select(prompt: &str, items: &[&str], default: usize) -> Result<String> {
    let selection = dialoguer::Select::new()
        .with_prompt(format!("  {prompt}"))
        .items(items)
        .default(default)
        .interact()
        .map_err(|e| crate::error::McmodError::Other(e.to_string()))?;
    Ok(items[selection].to_string())
}

fn prompt_multiselect(prompt: &str, items: &[&str]) -> Result<Vec<String>> {
    let defaults = vec![true; items.len()];
    let selections = dialoguer::MultiSelect::new()
        .with_prompt(format!("  {prompt}"))
        .items(items)
        .defaults(&defaults)
        .interact()
        .map_err(|e| crate::error::McmodError::Other(e.to_string()))?;
    Ok(selections.iter().map(|&i| items[i].to_string()).collect())
}

fn prompt_confirm(prompt: &str, default: bool) -> Result<bool> {
    let result = dialoguer::Confirm::new()
        .with_prompt(format!("  {prompt}"))
        .default(default)
        .interact()
        .map_err(|e| crate::error::McmodError::Other(e.to_string()))?;
    Ok(result)
}

fn create_run_options(project_dir: &Path) -> Result<()> {
    let run_dir = project_dir.join("run");
    crate::util::ensure_dir(&run_dir)?;
    crate::global_config::copy_options_to(&run_dir.join("options.txt"))
}

fn default_mod_name(mod_id: &str) -> String {
    mod_id
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => {
                    let mut s = c.to_uppercase().to_string();
                    s.extend(chars);
                    s
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
