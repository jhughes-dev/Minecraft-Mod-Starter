# Fabric Mod Boilerplate

A minimal boilerplate for creating Minecraft mods with Fabric.

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

- Renames all files and directories
- Enables Fabric API
- Creates mixin configuration
- Updates all references

### Manual Setup

1. Clone or copy this repository
1. Customize the mod (see checklist below)
1. Run `./gradlew build` to compile

## Manual Customization Checklist

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
1. Rename `ExampleMod.java` to your mod name
1. Update `MOD_ID` constant in your main class

### Assets

- Rename `src/main/resources/assets/modid/` to match your mod ID
- Replace `icon.png` with your 128x128 mod icon

## Fabric API

Fabric API is enabled by default after running the setup script.

If setting up manually, uncomment this line in `build.gradle`:

```gradle
modImplementation "net.fabricmc.fabric-api:fabric-api:${project.fabric_version}"
```

## Mixins

Mixin support is configured automatically by the setup script. The mixin config file is at `src/main/resources/<modid>.mixins.json`.

To add a mixin:

1. Create a class in your `mixin` package:

```java
package your.package.mixin;

import net.minecraft.client.gui.screen.TitleScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TitleScreen.class)
public class TitleScreenMixin {
    @Inject(at = @At("HEAD"), method = "init")
    private void onInit(CallbackInfo info) {
        System.out.println("Mixin injected!");
    }
}
```

1. Register it in `<modid>.mixins.json`:

```json
{
  "client": ["TitleScreenMixin"]
}
```

Use `"mixins"` for common mixins, `"client"` for client-only, `"server"` for server-only.

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

```text
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
