#!/usr/bin/env bash
# Multi-Loader Mod Setup Script
# Customizes this boilerplate for a new mod project (Fabric + NeoForge via Architectury)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FORCE=false
MC_VERSION_OVERRIDE=""

# Parse flags
while [[ "$1" == -* ]]; do
    case "$1" in
        -f|--force) FORCE=true; shift ;;
        --mc-version) MC_VERSION_OVERRIDE="$2"; shift 2 ;;
        *) shift ;;
    esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
NC='\033[0m'

print_header() {
    echo ""
    echo -e "${CYAN}================================${NC}"
    echo -e "${CYAN}  Multi-Loader Mod Setup${NC}"
    echo -e "${CYAN}  (Fabric + NeoForge)${NC}"
    echo -e "${CYAN}================================${NC}"
    echo ""
}

get_input() {
    local prompt="$1"
    local default="$2"
    local result
    read -p "$prompt [$default]: " result
    echo "${result:-$default}"
}

to_package_path() {
    echo "$1" | tr '.' '/'
}

to_pascal_case() {
    echo "$1" | sed -r 's/(^|_)([a-z])/\U\2/g'
}

escape_json_string() {
    printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' | tr -d '\r'
}

escape_toml_string() {
    printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' | tr -d '\r'
}

# --- Version Auto-Detection ---

fetch_latest_mc_version() {
    echo -e "${GRAY}  Querying latest stable Minecraft version...${NC}" >&2
    local result
    result=$(curl -sf --max-time 10 "https://meta.fabricmc.net/v2/versions/game" 2>/dev/null) || return 1
    echo "$result" | python3 -c "
import sys, json
data = json.load(sys.stdin)
for v in data:
    if v.get('stable'):
        print(v['version'])
        break
" 2>/dev/null
}

fetch_latest_loader_version() {
    echo -e "${GRAY}  Querying latest stable Fabric Loader version...${NC}" >&2
    local result
    result=$(curl -sf --max-time 10 "https://meta.fabricmc.net/v2/versions/loader" 2>/dev/null) || return 1
    echo "$result" | python3 -c "
import sys, json
data = json.load(sys.stdin)
for v in data:
    if v.get('stable'):
        print(v['version'])
        break
" 2>/dev/null
}

fetch_latest_fabric_api_version() {
    local mc_version="$1"
    echo -e "${GRAY}  Querying latest Fabric API for Minecraft ${mc_version}...${NC}" >&2
    local xml
    xml=$(curl -sf --max-time 10 "https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/maven-metadata.xml" 2>/dev/null) || return 1
    echo "$xml" | python3 -c "
import sys, xml.etree.ElementTree as ET
mc = '$mc_version'
tree = ET.parse(sys.stdin)
versions = [v.text for v in tree.findall('.//version') if v.text and v.text.endswith('+' + mc)]
if versions:
    print(versions[-1])
else:
    parts = mc.split('.')
    mc_short = '.'.join(parts[:2])
    versions = [v.text for v in tree.findall('.//version') if v.text and v.text.endswith('+' + mc_short)]
    if versions:
        print(versions[-1])
" 2>/dev/null
}

fetch_latest_neoforge_version() {
    local mc_version="$1"
    echo -e "${GRAY}  Querying latest NeoForge for Minecraft ${mc_version}...${NC}" >&2
    local xml
    xml=$(curl -sf --max-time 10 "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml" 2>/dev/null) || return 1
    echo "$xml" | python3 -c "
import sys, xml.etree.ElementTree as ET
mc = '$mc_version'
parts = mc.split('.')
neo_prefix = parts[1] + '.' + parts[2]
tree = ET.parse(sys.stdin)
versions = [v.text for v in tree.findall('.//version') if v.text and v.text.startswith(neo_prefix + '.')]
if versions:
    print(versions[-1])
" 2>/dev/null
}

# --- Main Script ---

print_header

# Gather input
MOD_ID=$(get_input "Mod ID (lowercase, no spaces, e.g., 'mymod')" "mymod")
MOD_NAME=$(get_input "Mod Display Name" "My Mod")
PACKAGE=$(get_input "Package name (e.g., 'io.github.username.mymod')" "io.github.yourname.$MOD_ID")
AUTHOR=$(get_input "Author name" "Your Name")
DESCRIPTION=$(get_input "Mod description" "A Minecraft mod")
LANGUAGE=$(get_input "Language (java/kotlin)" "java")
LANGUAGE=$(echo "$LANGUAGE" | tr '[:upper:]' '[:lower:]')
if [[ "$LANGUAGE" != "java" && "$LANGUAGE" != "kotlin" ]]; then
    echo -e "${RED}Error: Language must be 'java' or 'kotlin'${NC}"
    exit 1
fi
USE_KOTLIN=false
[[ "$LANGUAGE" == "kotlin" ]] && USE_KOTLIN=true

# Validate mod ID
if ! [[ "$MOD_ID" =~ ^[a-z][a-z0-9_]*$ ]]; then
    echo -e "${RED}Error: Mod ID must be lowercase, start with a letter, and contain only a-z, 0-9, _${NC}"
    exit 1
fi

# Fetch latest versions
echo -e "${CYAN}Fetching latest versions...${NC}"

# Read current defaults from gradle.properties
DEFAULT_MC_VERSION=$(grep -oP 'minecraft_version=\K.*' "$SCRIPT_DIR/gradle.properties" 2>/dev/null || echo "1.21.4")
DEFAULT_LOADER_VERSION=$(grep -oP 'fabric_loader_version=\K.*' "$SCRIPT_DIR/gradle.properties" 2>/dev/null || echo "0.16.9")
DEFAULT_FABRIC_VERSION=$(grep -oP '#?\s*fabric_api_version=\K.*' "$SCRIPT_DIR/gradle.properties" 2>/dev/null || echo "0.111.0+1.21.4")
DEFAULT_NEOFORGE_VERSION=$(grep -oP 'neoforge_version=\K.*' "$SCRIPT_DIR/gradle.properties" 2>/dev/null || echo "21.4.156")

FETCHED_MC_VERSION=$(fetch_latest_mc_version 2>&1 | tail -1) || FETCHED_MC_VERSION=""
FETCHED_LOADER_VERSION=$(fetch_latest_loader_version 2>&1 | tail -1) || FETCHED_LOADER_VERSION=""

# Use override > fetched > default
if [[ -n "$MC_VERSION_OVERRIDE" ]]; then
    MC_VERSION="$MC_VERSION_OVERRIDE"
elif [[ -n "$FETCHED_MC_VERSION" ]]; then
    MC_VERSION="$FETCHED_MC_VERSION"
else
    MC_VERSION="$DEFAULT_MC_VERSION"
fi

LOADER_VERSION="${FETCHED_LOADER_VERSION:-$DEFAULT_LOADER_VERSION}"

FETCHED_FABRIC_VERSION=$(fetch_latest_fabric_api_version "$MC_VERSION" 2>&1 | tail -1) || FETCHED_FABRIC_VERSION=""
FABRIC_VERSION="${FETCHED_FABRIC_VERSION:-$DEFAULT_FABRIC_VERSION}"

FETCHED_NEOFORGE_VERSION=$(fetch_latest_neoforge_version "$MC_VERSION" 2>&1 | tail -1) || FETCHED_NEOFORGE_VERSION=""
NEOFORGE_VERSION="${FETCHED_NEOFORGE_VERSION:-$DEFAULT_NEOFORGE_VERSION}"

# Derive NeoForge major version for dependency range
NEOFORGE_MAJOR=$(echo "$NEOFORGE_VERSION" | cut -d. -f1-2)

echo ""
echo -e "${YELLOW}Configuration:${NC}"
echo "  Mod ID:      $MOD_ID"
echo "  Mod Name:    $MOD_NAME"
echo "  Package:     $PACKAGE"
echo "  Author:      $AUTHOR"
echo "  Description: $DESCRIPTION"
echo "  Language:    $LANGUAGE"
echo ""
echo -e "${YELLOW}Versions:${NC}"
echo "  Minecraft:     $MC_VERSION"
echo "  Fabric Loader: $LOADER_VERSION"
echo "  Fabric API:    $FABRIC_VERSION"
echo "  NeoForge:      $NEOFORGE_VERSION"
echo ""

if [[ "$FORCE" != "true" ]]; then
    read -p "Proceed with setup? (y/N): " confirm
    if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
        echo -e "${YELLOW}Setup cancelled.${NC}"
        exit 0
    fi
fi

echo ""
echo -e "${GREEN}Setting up mod...${NC}"

# Derive class name
CLASS_NAME="$(to_pascal_case "$MOD_ID")Mod"

PACKAGE_PATH=$(to_package_path "$PACKAGE")

# --- Paths ---
SRC_LANG="java"
[[ "$USE_KOTLIN" == "true" ]] && SRC_LANG="kotlin"
COMMON_SRC_DIR="$SCRIPT_DIR/common/src/main/$SRC_LANG"
COMMON_RESOURCES_DIR="$SCRIPT_DIR/common/src/main/resources"
FABRIC_SRC_DIR="$SCRIPT_DIR/fabric/src/main/$SRC_LANG"
FABRIC_RESOURCES_DIR="$SCRIPT_DIR/fabric/src/main/resources"
NEOFORGE_SRC_DIR="$SCRIPT_DIR/neoforge/src/main/$SRC_LANG"
NEOFORGE_RESOURCES_DIR="$SCRIPT_DIR/neoforge/src/main/resources"

# Old paths (always in java dir, from template defaults)
COMMON_JAVA_DIR="$SCRIPT_DIR/common/src/main/java"
FABRIC_JAVA_DIR="$SCRIPT_DIR/fabric/src/main/java"
NEOFORGE_JAVA_DIR="$SCRIPT_DIR/neoforge/src/main/java"

OLD_COMMON_PACKAGE="$COMMON_JAVA_DIR/io/github/yourname/modid"
NEW_COMMON_PACKAGE="$COMMON_SRC_DIR/$PACKAGE_PATH"
OLD_FABRIC_PACKAGE="$FABRIC_JAVA_DIR/io/github/yourname/modid/fabric"
NEW_FABRIC_PACKAGE="$FABRIC_SRC_DIR/$PACKAGE_PATH/fabric"
OLD_NEOFORGE_PACKAGE="$NEOFORGE_JAVA_DIR/io/github/yourname/modid/neoforge"
NEW_NEOFORGE_PACKAGE="$NEOFORGE_SRC_DIR/$PACKAGE_PATH/neoforge"

# Helper to clean empty parent dirs
clean_empty_parents() {
    local path="$1"
    local stop_at="$2"
    local parent_path="$(dirname "$path")"
    while [ "$parent_path" != "$stop_at" ]; do
        if [ -z "$(ls -A "$parent_path" 2>/dev/null)" ]; then
            rmdir "$parent_path"
            parent_path="$(dirname "$parent_path")"
        else
            break
        fi
    done
}

# 1. Update gradle.properties
echo -e "${GRAY}  Updating gradle.properties...${NC}"
sed -i.bak "s/minecraft_version=.*/minecraft_version=$MC_VERSION/" "$SCRIPT_DIR/gradle.properties"
sed -i.bak "s/fabric_loader_version=.*/fabric_loader_version=$LOADER_VERSION/" "$SCRIPT_DIR/gradle.properties"
sed -i.bak "s|# fabric_api_version=.*|fabric_api_version=$FABRIC_VERSION|" "$SCRIPT_DIR/gradle.properties"
sed -i.bak "s/neoforge_version=.*/neoforge_version=$NEOFORGE_VERSION/" "$SCRIPT_DIR/gradle.properties"
sed -i.bak "s/# mod_language=.*/mod_language=$LANGUAGE/" "$SCRIPT_DIR/gradle.properties"
if [[ "$USE_KOTLIN" == "true" ]]; then
    sed -i.bak 's/# kotlin_version=/kotlin_version=/' "$SCRIPT_DIR/gradle.properties"
fi
sed -i.bak "s/maven_group=.*/maven_group=$PACKAGE/" "$SCRIPT_DIR/gradle.properties"
sed -i.bak "s/archives_base_name=.*/archives_base_name=$MOD_ID/" "$SCRIPT_DIR/gradle.properties"
sed -i.bak "s/mod_name=.*/mod_name=$MOD_NAME/" "$SCRIPT_DIR/gradle.properties"
rm -f "$SCRIPT_DIR/gradle.properties.bak"

# 3. Update settings.gradle
echo -e "${GRAY}  Updating settings.gradle...${NC}"
sed -i.bak "s/rootProject\.name = \"modid\"/rootProject.name = \"$MOD_ID\"/" "$SCRIPT_DIR/settings.gradle"
rm -f "$SCRIPT_DIR/settings.gradle.bak"

# 4. Enable Fabric API in fabric/build.gradle
echo -e "${GRAY}  Enabling Fabric API in fabric/build.gradle...${NC}"
sed -i.bak 's|// modApi "net.fabricmc.fabric-api:fabric-api|modApi "net.fabricmc.fabric-api:fabric-api|' "$SCRIPT_DIR/fabric/build.gradle"
rm -f "$SCRIPT_DIR/fabric/build.gradle.bak"

# 5. Create common module source
echo -e "${GRAY}  Creating common module source...${NC}"
mkdir -p "$NEW_COMMON_PACKAGE"

if [[ "$USE_KOTLIN" == "true" ]]; then
    cat > "$NEW_COMMON_PACKAGE/$CLASS_NAME.kt" << EOF
package $PACKAGE

import org.slf4j.LoggerFactory

object $CLASS_NAME {
    const val MOD_ID = "$MOD_ID"
    val LOGGER = LoggerFactory.getLogger(MOD_ID)

    fun init() {
        LOGGER.info("Initializing $MOD_NAME")
    }
}
EOF
else
    cat > "$NEW_COMMON_PACKAGE/$CLASS_NAME.java" << EOF
package $PACKAGE;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class $CLASS_NAME {
    public static final String MOD_ID = "$MOD_ID";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    public static void init() {
        LOGGER.info("Initializing $MOD_NAME");
    }
}
EOF
fi

# Remove old common source
if [ -d "$OLD_COMMON_PACKAGE" ] && [ "$OLD_COMMON_PACKAGE" != "$NEW_COMMON_PACKAGE" ]; then
    rm -rf "$OLD_COMMON_PACKAGE"
    clean_empty_parents "$OLD_COMMON_PACKAGE" "$COMMON_JAVA_DIR"
fi
if [[ "$USE_KOTLIN" == "true" ]] && [ -d "$COMMON_JAVA_DIR" ]; then
    if [ -z "$(find "$COMMON_JAVA_DIR" -type f 2>/dev/null)" ]; then rm -rf "$COMMON_JAVA_DIR"; fi
fi

# Move common assets
OLD_COMMON_ASSETS="$COMMON_RESOURCES_DIR/assets/modid"
NEW_COMMON_ASSETS="$COMMON_RESOURCES_DIR/assets/$MOD_ID"
if [ -d "$OLD_COMMON_ASSETS" ] && [ "$OLD_COMMON_ASSETS" != "$NEW_COMMON_ASSETS" ]; then
    mv "$OLD_COMMON_ASSETS" "$NEW_COMMON_ASSETS"
fi

# 6. Create Fabric module source
echo -e "${GRAY}  Creating Fabric module source...${NC}"
mkdir -p "$NEW_FABRIC_PACKAGE"
mkdir -p "$(dirname "$NEW_FABRIC_PACKAGE")/mixin"

if [[ "$USE_KOTLIN" == "true" ]]; then
    cat > "$NEW_FABRIC_PACKAGE/${CLASS_NAME}Fabric.kt" << EOF
package $PACKAGE.fabric

import $PACKAGE.$CLASS_NAME
import net.fabricmc.api.ModInitializer

class ${CLASS_NAME}Fabric : ModInitializer {
    override fun onInitialize() {
        ${CLASS_NAME}.init()
    }
}
EOF
else
    cat > "$NEW_FABRIC_PACKAGE/${CLASS_NAME}Fabric.java" << EOF
package $PACKAGE.fabric;

import $PACKAGE.$CLASS_NAME;
import net.fabricmc.api.ModInitializer;

public class ${CLASS_NAME}Fabric implements ModInitializer {
    @Override
    public void onInitialize() {
        ${CLASS_NAME}.init();
    }
}
EOF
fi

cat > "$(dirname "$NEW_FABRIC_PACKAGE")/mixin/package-info.java" << EOF
/** Mixin classes for $MOD_NAME */
package $PACKAGE.mixin;
EOF

# Remove old fabric source
if [ -d "$OLD_FABRIC_PACKAGE" ] && [ "$OLD_FABRIC_PACKAGE" != "$NEW_FABRIC_PACKAGE" ]; then
    rm -rf "$OLD_FABRIC_PACKAGE"
    clean_empty_parents "$OLD_FABRIC_PACKAGE" "$FABRIC_JAVA_DIR"
fi
if [[ "$USE_KOTLIN" == "true" ]] && [ -d "$OLD_FABRIC_PACKAGE" ]; then
    rm -rf "$OLD_FABRIC_PACKAGE" 2>/dev/null || true
fi

# Escape user input for JSON/TOML
SAFE_MOD_NAME=$(escape_json_string "$MOD_NAME")
SAFE_DESCRIPTION=$(escape_json_string "$DESCRIPTION")
SAFE_AUTHOR=$(escape_json_string "$AUTHOR")
SAFE_MOD_NAME_TOML=$(escape_toml_string "$MOD_NAME")
SAFE_DESCRIPTION_TOML=$(escape_toml_string "$DESCRIPTION")
SAFE_AUTHOR_TOML=$(escape_toml_string "$AUTHOR")

# Create fabric.mod.json
cat > "$FABRIC_RESOURCES_DIR/fabric.mod.json" << EOF
{
  "schemaVersion": 1,
  "id": "$MOD_ID",
  "version": "\${version}",
  "name": "$SAFE_MOD_NAME",
  "description": "$SAFE_DESCRIPTION",
  "authors": ["$SAFE_AUTHOR"],
  "contact": {
    "homepage": "https://github.com/yourname/$MOD_ID",
    "sources": "https://github.com/yourname/$MOD_ID"
  },
  "license": "MIT",
  "icon": "assets/$MOD_ID/icon.png",
  "environment": "*",
  "entrypoints": {
    "main": ["$PACKAGE.fabric.${CLASS_NAME}Fabric"]
  },
  "mixins": ["$MOD_ID.mixins.json"],
  "depends": {
    "fabricloader": ">=$LOADER_VERSION",
    "minecraft": "~$MC_VERSION",
    "java": ">=21",
    "fabric-api": "*"
  }
}
EOF

# Create mixin config for fabric
cat > "$FABRIC_RESOURCES_DIR/$MOD_ID.mixins.json" << EOF
{
  "required": true,
  "minVersion": "0.8",
  "package": "$PACKAGE.mixin",
  "compatibilityLevel": "JAVA_21",
  "mixins": [],
  "client": [],
  "server": [],
  "injectors": {
    "defaultRequire": 1
  }
}
EOF

# 7. Create NeoForge module source
echo -e "${GRAY}  Creating NeoForge module source...${NC}"
mkdir -p "$NEW_NEOFORGE_PACKAGE"

if [[ "$USE_KOTLIN" == "true" ]]; then
    cat > "$NEW_NEOFORGE_PACKAGE/${CLASS_NAME}NeoForge.kt" << EOF
package $PACKAGE.neoforge

import $PACKAGE.$CLASS_NAME
import net.neoforged.bus.api.IEventBus
import net.neoforged.fml.common.Mod

@Mod(${CLASS_NAME}.MOD_ID)
class ${CLASS_NAME}NeoForge(modEventBus: IEventBus) {
    init {
        ${CLASS_NAME}.init()
    }
}
EOF
else
    cat > "$NEW_NEOFORGE_PACKAGE/${CLASS_NAME}NeoForge.java" << EOF
package $PACKAGE.neoforge;

import $PACKAGE.$CLASS_NAME;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;

@Mod(${CLASS_NAME}.MOD_ID)
public class ${CLASS_NAME}NeoForge {
    public ${CLASS_NAME}NeoForge(IEventBus modEventBus) {
        ${CLASS_NAME}.init();
    }
}
EOF
fi

# Remove old neoforge source
if [ -d "$OLD_NEOFORGE_PACKAGE" ] && [ "$OLD_NEOFORGE_PACKAGE" != "$NEW_NEOFORGE_PACKAGE" ]; then
    rm -rf "$OLD_NEOFORGE_PACKAGE"
    clean_empty_parents "$OLD_NEOFORGE_PACKAGE" "$NEOFORGE_JAVA_DIR"
fi
if [[ "$USE_KOTLIN" == "true" ]] && [ -d "$NEOFORGE_JAVA_DIR" ]; then
    if [ -z "$(find "$NEOFORGE_JAVA_DIR" -type f 2>/dev/null)" ]; then rm -rf "$NEOFORGE_JAVA_DIR"; fi
fi

# Create neoforge.mods.toml
mkdir -p "$NEOFORGE_RESOURCES_DIR/META-INF"
cat > "$NEOFORGE_RESOURCES_DIR/META-INF/neoforge.mods.toml" << EOF
modLoader = "javafml"
loaderVersion = "[4,)"
license = "MIT"

[[mods]]
modId = "$MOD_ID"
version = "\${version}"
displayName = "$SAFE_MOD_NAME_TOML"
description = "$SAFE_DESCRIPTION_TOML"
authors = "$SAFE_AUTHOR_TOML"
logoFile = "assets/$MOD_ID/icon.png"

[[dependencies.$MOD_ID]]
modId = "neoforge"
type = "required"
versionRange = "[$NEOFORGE_MAJOR,)"
ordering = "NONE"
side = "BOTH"

[[dependencies.$MOD_ID]]
modId = "minecraft"
type = "required"
versionRange = "[$MC_VERSION,)"
ordering = "NONE"
side = "BOTH"
EOF

# 8. Update LICENSE copyright
echo -e "${GRAY}  Updating LICENSE...${NC}"
YEAR=$(date +%Y)
sed -i.bak "s/Copyright (c) [0-9]* Your Name/Copyright (c) $YEAR $AUTHOR/" "$SCRIPT_DIR/LICENSE"
rm -f "$SCRIPT_DIR/LICENSE.bak"

echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Replace common/src/main/resources/assets/$MOD_ID/icon.png.txt with your mod icon (128x128 PNG)"
echo "  2. Run './gradlew build' to verify the setup"
echo "  3. Open in your IDE (IntelliJ IDEA recommended)"
echo ""
echo -e "${YELLOW}Project structure:${NC}"
echo "  common/   - Shared code (both loaders)"
echo "  fabric/   - Fabric-specific code"
echo "  neoforge/ - NeoForge-specific code"
echo ""
echo -e "${YELLOW}Build outputs:${NC}"
echo "  fabric/build/libs/$MOD_ID-*.jar"
echo "  neoforge/build/libs/$MOD_ID-*.jar"
echo ""
echo -e "${GRAY}Optional: Delete this setup script after use${NC}"
