# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repo contains two things:

1. **Mod boilerplate** — A multi-loader Minecraft mod template using Architectury (Fabric + NeoForge, MC 1.21.4, Java 21)
2. **`mcmod` CLI** — A Rust CLI tool (`cli/`) that scaffolds new mod projects from the boilerplate templates

## Build Commands

### Mod boilerplate (Gradle)

```bash
./gradlew build                  # Build both Fabric and NeoForge JARs
./gradlew :fabric:build          # Build only Fabric
./gradlew :neoforge:build        # Build only NeoForge
./gradlew :fabric:runClient      # Run Minecraft client with Fabric
./gradlew :neoforge:runClient    # Run Minecraft client with NeoForge
```

Build outputs: `fabric/build/libs/` and `neoforge/build/libs/`. No test or lint tasks configured.

### CLI tool (Rust)

```bash
cd cli && cargo build            # Build the mcmod CLI
cd cli && cargo test             # Run Rust unit tests (template rendering, config roundtrip)
```

The built binary is at `cli/target/debug/mcmod.exe` (Windows) or `cli/target/debug/mcmod`.

### Integration tests (PowerShell)

```powershell
.\test\run-tests.ps1                       # Run all golden-file tests (java + kotlin)
.\test\run-tests.ps1 -Language java        # Test only java scaffolding
.\test\run-tests.ps1 -Language kotlin      # Test only kotlin scaffolding
.\test\run-tests.ps1 -TestCI              # Also test CI workflow generation
```

These run `setup.ps1` in temp directories and diff output against `test/golden/` files. Version numbers are normalized before comparison.

## Architecture

### Mod boilerplate (Architectury multi-loader)

Three-module Gradle project:

- **`common/`** — Platform-agnostic shared code. Contains core mod logic (`ExampleMod.init()`).
- **`fabric/`** — Fabric entry point (`ModInitializer`). Delegates to common. Metadata in `fabric.mod.json`.
- **`neoforge/`** — NeoForge entry point (`@Mod` + `IEventBus`). Delegates to common. Metadata in `META-INF/neoforge.mods.toml`.

All shared game logic goes in `common/`. Both platform modules call into common's `init()` method.

### CLI tool (`cli/`)

Rust binary using `clap` for argument parsing and `dialoguer` for interactive prompts. Structure:

- **`src/main.rs`** — CLI entry point, defines `Commands` enum: `Init`, `Add`, `Update`, `Config`
- **`src/commands/init.rs`** — Scaffolds a new project: gathers inputs (interactive or flags), fetches latest versions from APIs, renders templates, writes files
- **`src/commands/add.rs`** — Adds features (fabric, neoforge, ci, kotlin) to existing projects. Reads/updates `mcmod.toml`
- **`src/commands/update.rs`** — Self-update from GitHub releases
- **`src/commands/config.rs`** — Global config management (`mcmod config set/get/list`)
- **`src/template.rs`** — `include_str!`/`include_bytes!` for all templates, `render()` does `{{placeholder}}` substitution
- **`src/config.rs`** — `McmodConfig` (per-project `mcmod.toml`): mod info, loaders, features, versions
- **`src/global_config.rs`** — Global CLI preferences stored in `%APPDATA%/mcmod/config.toml` (Windows) or `~/.config/mcmod/config.toml`
- **`src/versions.rs`** — Fetches latest Minecraft, Fabric Loader, Fabric API, and NeoForge versions from their respective Maven/API endpoints
- **`src/gradle.rs`** — Helpers for modifying `settings.gradle` and `gradle.properties`

Templates live in `cli/templates/` and are embedded into the binary at compile time via `include_str!`/`include_bytes!`. Changing a template file requires recompiling the CLI.

### Key configuration

- **`gradle.properties`** — Mod ID, versions, group, and dependency versions
- **`mcmod.toml`** — Per-project config written by the CLI, tracks mod info, enabled loaders/features, and versions. Used by `mcmod add` to modify existing projects.
- **Root `build.gradle`** — Applies Architectury plugin and Loom; sets Java 21, official Minecraft mappings. Conditionally applies Kotlin plugin if `mod_language=kotlin`.

### Setup scripts (legacy)

`setup.ps1` and `setup.sh` are the original setup scripts (before the Rust CLI). The PowerShell script is still used by integration tests.
