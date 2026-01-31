# Multi-Loader Mod Boilerplate

A minimal boilerplate for creating Minecraft mods targeting both **Fabric** and **NeoForge** using [Architectury](https://docs.architectury.dev/).

## Requirements

- Java 21 or higher
- Minecraft 1.21.4

## Quick Start

### Automated Setup (Recommended)

Run the setup script to automatically configure your mod:

**Windows (PowerShell):**

```powershell
.\setup.ps1
```

**Linux/macOS/Git Bash:**

```bash
./setup.sh
```

The script will prompt for:

- **Mod ID** - lowercase identifier (e.g., `mymod`)
- **Mod Name** - display name (e.g., `My Awesome Mod`)
- **Package** - Java package (e.g., `io.github.yourname.mymod`)
- **Author** - your name
- **Description** - short description

The script automatically:

- Fetches latest Minecraft, Fabric, NeoForge, and Gradle versions
- Renames all files and directories across all modules
- Enables Fabric API
- Creates mixin configuration
- Updates all references

Non-interactive mode:

```powershell
.\setup.ps1 -ModId "mymod" -ModName "My Mod" -Package "io.github.me.mymod" -Author "Me" -Description "My mod" -Force
```

## Project Structure

```text
├── common/                         # Shared code (both loaders)
│   └── src/main/
│       ├── java/                   # Platform-agnostic mod logic
│       └── resources/assets/       # Shared assets (textures, etc.)
├── fabric/                         # Fabric-specific code
│   └── src/main/
│       ├── java/                   # Fabric entry point
│       └── resources/
│           ├── fabric.mod.json     # Fabric mod metadata
│           └── *.mixins.json       # Mixin configuration
├── neoforge/                       # NeoForge-specific code
│   └── src/main/
│       ├── java/                   # NeoForge entry point
│       └── resources/META-INF/
│           └── neoforge.mods.toml  # NeoForge mod metadata
├── build.gradle                    # Root build configuration
├── settings.gradle                 # Multi-module settings
└── gradle.properties               # Version configuration
```

## Build Commands

```bash
# Build both Fabric and NeoForge JARs
./gradlew build

# Build only Fabric
./gradlew :fabric:build

# Build only NeoForge
./gradlew :neoforge:build

# Run Minecraft client with Fabric
./gradlew :fabric:runClient

# Run Minecraft client with NeoForge
./gradlew :neoforge:runClient

# Clean build artifacts
./gradlew clean
```

Build outputs:

- `fabric/build/libs/<modid>-<version>.jar`
- `neoforge/build/libs/<modid>-<version>.jar`

## Adding Code

- **Shared logic** goes in `common/` - this code runs on both loaders
- **Fabric-specific** code goes in `fabric/` (e.g., Fabric API usage)
- **NeoForge-specific** code goes in `neoforge/` (e.g., NeoForge events)

The common module's `init()` method is called by both platform entry points.

## Mixins

Mixin support is configured for Fabric. The mixin config file is at `fabric/src/main/resources/<modid>.mixins.json`.

To add a mixin, create a class in your `mixin` package under `fabric/src/main/java/` and register it in the mixins JSON.

## Fabric API

Fabric API is enabled by default after running the setup script.

If setting up manually, uncomment this line in `fabric/build.gradle`:

```gradle
modApi "net.fabricmc.fabric-api:fabric-api:${rootProject.fabric_api_version}"
```

And uncomment `fabric_api_version` in `gradle.properties`.

## Resources

- [Architectury Documentation](https://docs.architectury.dev/)
- [Fabric Documentation](https://docs.fabricmc.net/)
- [NeoForge Documentation](https://docs.neoforged.net/)

## License

MIT License - see [LICENSE](LICENSE)
