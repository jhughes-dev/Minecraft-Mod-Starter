# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Multi-loader Minecraft mod boilerplate using the **Architectury** framework. Targets Minecraft 1.21.4 with Java 21. Supports both **Fabric** and **NeoForge** loaders from a single codebase, with optional Kotlin support.

## Build Commands

```bash
./gradlew build                  # Build both Fabric and NeoForge JARs
./gradlew :fabric:build          # Build only Fabric
./gradlew :neoforge:build        # Build only NeoForge
./gradlew :fabric:runClient      # Run Minecraft client with Fabric
./gradlew :neoforge:runClient    # Run Minecraft client with NeoForge
```

Build outputs: `fabric/build/libs/` and `neoforge/build/libs/`. No test or lint tasks are configured.

## Architecture

Three-module Gradle project using Architectury multi-loader pattern:

- **`common/`** — Platform-agnostic shared code. Both loaders depend on this. Contains the core mod logic (e.g., `ExampleMod.init()`).
- **`fabric/`** — Fabric entry point implementing `ModInitializer`. Delegates to common. Metadata in `fabric.mod.json`.
- **`neoforge/`** — NeoForge entry point using `@Mod` annotation with `IEventBus`. Delegates to common. Metadata in `META-INF/neoforge.mods.toml`.

All shared game logic goes in `common/`. Platform-specific code (entry points, platform APIs) goes in the respective loader module. Both platform modules call into common's `init()` method.

## Key Configuration

- **`gradle.properties`** — Mod ID, versions, group, and dependency versions are all defined here.
- **Root `build.gradle`** — Applies Architectury plugin and Loom to all subprojects; sets Java 21, official Minecraft mappings.
- **Shadow JAR** — Both `fabric/` and `neoforge/` use the Shadow plugin to bundle dependencies.

## Setup Scripts

`setup.ps1` (Windows) and `setup.sh` (Linux/macOS) automate project initialization: renaming packages, setting mod ID, auto-detecting latest Minecraft/Fabric/NeoForge/Gradle versions, optionally enabling Fabric API and mixins.
