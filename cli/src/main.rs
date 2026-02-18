mod commands;
mod config;
mod error;
mod global_config;
mod gradle;
mod template;
mod util;
mod versions;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "mcmod", about = "CLI tool for scaffolding multi-loader Minecraft mods")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Minecraft mod project
    Init {
        /// Project directory (default: current directory)
        #[arg(long, default_value = ".")]
        dir: PathBuf,

        /// Mod ID (lowercase + underscores)
        #[arg(long)]
        mod_id: Option<String>,

        /// Display name for the mod
        #[arg(long, alias = "name")]
        mod_name: Option<String>,

        /// Java package name
        #[arg(long)]
        package: Option<String>,

        /// Author name
        #[arg(long)]
        author: Option<String>,

        /// Mod description
        #[arg(long)]
        description: Option<String>,

        /// Language: java or kotlin
        #[arg(long)]
        language: Option<String>,

        /// Loaders to enable (can be specified multiple times)
        #[arg(long = "loader")]
        loaders: Vec<String>,

        /// Enable GitHub Actions CI
        #[arg(long)]
        ci: Option<bool>,

        /// Skip online version fetching, use defaults
        #[arg(long)]
        offline: bool,
    },

    /// Add a feature to an existing project
    Add {
        /// Feature to add: fabric, neoforge, ci, kotlin
        feature: String,

        /// Project directory (default: current directory)
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },

    /// Update mcmod to the latest version
    Update,

    /// Manage global CLI preferences
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set a global preference (e.g., mcmod config set author "Jane")
    Set { key: String, value: String },
    /// Get a global preference value
    Get { key: String },
    /// List all global preferences
    List,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init {
            dir,
            mod_id,
            mod_name,
            package,
            author,
            description,
            language,
            loaders,
            ci,
            offline,
        } => commands::init::run(commands::init::InitOptions {
            dir,
            mod_id,
            mod_name,
            package,
            author,
            description,
            language,
            loaders,
            ci,
            offline,
        }),
        Commands::Add { feature, dir } => commands::add::run(&feature, &dir),
        Commands::Update => commands::update::run(),
        Commands::Config { action } => match action {
            ConfigCommands::Set { key, value } => commands::config::run_set(&key, &value),
            ConfigCommands::Get { key } => commands::config::run_get(&key),
            ConfigCommands::List => commands::config::run_list(),
        },
    };

    if let Err(e) = result {
        eprintln!("{}", format!("\n  Error: {e}\n").red().bold());
        process::exit(1);
    }
}
