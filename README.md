# Fabric Mod Boilerplate

A minimal boilerplate for creating Minecraft mods with Fabric.

## Requirements

- Java 21 or higher
- Minecraft 1.21.4

## Quick Start

1. Clone or copy this repository
2. Customize the mod (see below)
3. Run `./gradlew build` to compile

## Customization Checklist

When starting a new mod, update these files:

### gradle.properties
```properties
maven_group=io.github.yourname      # Your package group
archives_base_name=yourmodid        # Your mod ID
mod_version=1.0.0                   # Your mod version
```

### fabric.mod.json
- `id`: Your mod ID (lowercase, no spaces)
- `name`: Display name
- `description`: Mod description
- `authors`: Your name
- `contact`: Your URLs
- `entrypoints.main`: Update package path

### Source Code
1. Rename package `io.github.yourname.modid` to match your `maven_group` + mod ID
2. Rename `ExampleMod.java` to your mod name
3. Update `MOD_ID` constant in your main class

### Assets
- Rename `src/main/resources/assets/modid/` to match your mod ID
- Replace `icon.png` with your 128x128 mod icon

## Adding Fabric API

To use Fabric API, uncomment these lines:

**gradle.properties:**
```properties
fabric_version=0.111.0+1.21.4
```

**build.gradle:**
```gradle
modImplementation "net.fabricmc.fabric-api:fabric-api:${project.fabric_version}"
```

## Adding Mixins

1. Create `src/main/resources/modid.mixins.json`:
```json
{
    "required": true,
    "package": "io.github.yourname.modid.mixin",
    "compatibilityLevel": "JAVA_21",
    "mixins": [],
    "client": [],
    "injectors": {
        "defaultRequire": 1
    }
}
```

2. Add to `fabric.mod.json`:
```json
"mixins": ["modid.mixins.json"]
```

3. Create mixin classes in `src/main/java/io/github/yourname/modid/mixin/`

## Build Commands

```bash
# Build the mod
./gradlew build

# Run Minecraft client with mod
./gradlew runClient

# Run Minecraft server with mod
./gradlew runServer

# Clean build artifacts
./gradlew clean
```

## Project Structure

```
├── src/main/
│   ├── java/                    # Java source code
│   └── resources/
│       ├── fabric.mod.json      # Mod metadata
│       └── assets/modid/        # Mod assets (textures, etc.)
├── build.gradle                 # Build configuration
├── gradle.properties            # Version configuration
└── settings.gradle              # Gradle settings
```

## Resources

- [Fabric Documentation](https://docs.fabricmc.net/)
- [Fabric Discord](https://discord.gg/v6v4pMv)
- [Fabric API on Modrinth](https://modrinth.com/mod/fabric-api)

## License

MIT License - see [LICENSE](LICENSE)
