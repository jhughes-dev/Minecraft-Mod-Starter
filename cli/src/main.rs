mod commands;
mod config;
mod error;
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
    };

    if let Err(e) = result {
        eprintln!("{}", format!("\n  Error: {e}\n").red().bold());
        process::exit(1);
    }
}
