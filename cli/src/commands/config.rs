use crate::error::Result;
use crate::global_config::{self, GlobalConfig};
use colored::Colorize;

pub fn run_set(key: &str, value: &str) -> Result<()> {
    let mut config = GlobalConfig::load()?;
    config.set(key, value)?;
    println!(
        "{}",
        format!("  Set {key} = {value}").green()
    );
    Ok(())
}

pub fn run_get(key: &str) -> Result<()> {
    let config = GlobalConfig::load()?;
    match config.get(key) {
        Some(value) => println!("  {value}"),
        None => println!("  {}", "(not set)".dimmed()),
    }
    Ok(())
}

pub fn run_list() -> Result<()> {
    let config = GlobalConfig::load()?;
    let dir = global_config::global_config_dir()?;

    println!("{}", "\n  mcmod global config\n".bold().cyan());
    println!("  {}", format!("Config directory: {}", dir.display()).dimmed());
    println!();

    let entries = config.list();
    let mut current_section = "";

    for (section, key, value) in &entries {
        if *section != current_section {
            if !current_section.is_empty() {
                println!();
            }
            println!("  {}", format!("[{section}]").bold());
            current_section = section;
        }
        println!("  {:<22} {}", format!("{key}:"), value);
    }
    println!();
    Ok(())
}
