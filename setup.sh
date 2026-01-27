#!/usr/bin/env bash
# Fabric Mod Setup Script
# Customizes this boilerplate for a new mod project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FORCE=false

# Parse flags
while [[ "$1" == -* ]]; do
    case "$1" in
        -f|--force) FORCE=true; shift ;;
        *) shift ;;
    esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

print_header() {
    echo ""
    echo -e "${CYAN}================================${NC}"
    echo -e "${CYAN}  Fabric Mod Setup Script${NC}"
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

print_header

# Gather input
MOD_ID=$(get_input "Mod ID (lowercase, no spaces, e.g., 'mymod')" "mymod")
MOD_NAME=$(get_input "Mod Display Name" "My Mod")
PACKAGE=$(get_input "Package name (e.g., 'io.github.username.mymod')" "io.github.yourname.$MOD_ID")
AUTHOR=$(get_input "Author name" "Your Name")
DESCRIPTION=$(get_input "Mod description" "A Fabric mod for Minecraft")

# Validate mod ID
if ! [[ "$MOD_ID" =~ ^[a-z][a-z0-9_]*$ ]]; then
    echo -e "${RED}Error: Mod ID must be lowercase, start with a letter, and contain only a-z, 0-9, _${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Configuration:${NC}"
echo "  Mod ID:      $MOD_ID"
echo "  Mod Name:    $MOD_NAME"
echo "  Package:     $PACKAGE"
echo "  Author:      $AUTHOR"
echo "  Description: $DESCRIPTION"
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

# Paths
SRC_DIR="$SCRIPT_DIR/src"
MAIN_DIR="$SRC_DIR/main"
JAVA_DIR="$MAIN_DIR/java"
RESOURCES_DIR="$MAIN_DIR/resources"
ASSETS_DIR="$RESOURCES_DIR/assets"

OLD_PACKAGE_PATH="$JAVA_DIR/io/github/yourname/modid"
NEW_PACKAGE_PATH="$JAVA_DIR/$(to_package_path "$PACKAGE")"
OLD_ASSETS_DIR="$ASSETS_DIR/modid"
NEW_ASSETS_DIR="$ASSETS_DIR/$MOD_ID"

# Derive class name
CLASS_NAME="$(to_pascal_case "$MOD_ID")Mod"

# 1. Update gradle.properties
echo -e "${GRAY}  Updating gradle.properties...${NC}"
sed -i.bak "s/maven_group=.*/maven_group=$PACKAGE/" "$SCRIPT_DIR/gradle.properties"
sed -i.bak "s/archives_base_name=.*/archives_base_name=$MOD_ID/" "$SCRIPT_DIR/gradle.properties"
sed -i.bak "s/# fabric_version=/fabric_version=/" "$SCRIPT_DIR/gradle.properties"
rm -f "$SCRIPT_DIR/gradle.properties.bak"

# 2. Update build.gradle - enable Fabric API
echo -e "${GRAY}  Enabling Fabric API in build.gradle...${NC}"
sed -i.bak 's|// modImplementation "net.fabricmc.fabric-api:fabric-api|modImplementation "net.fabricmc.fabric-api:fabric-api|' "$SCRIPT_DIR/build.gradle"
rm -f "$SCRIPT_DIR/build.gradle.bak"

# 3. Create mixin config file
echo -e "${GRAY}  Creating mixin configuration...${NC}"
cat > "$RESOURCES_DIR/$MOD_ID.mixins.json" << EOF
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

# 4. Update fabric.mod.json
echo -e "${GRAY}  Updating fabric.mod.json...${NC}"
cat > "$RESOURCES_DIR/fabric.mod.json" << EOF
{
  "schemaVersion": 1,
  "id": "$MOD_ID",
  "version": "\${version}",
  "name": "$MOD_NAME",
  "description": "$DESCRIPTION",
  "authors": ["$AUTHOR"],
  "contact": {
    "homepage": "https://github.com/yourname/$MOD_ID",
    "sources": "https://github.com/yourname/$MOD_ID"
  },
  "license": "MIT",
  "icon": "assets/$MOD_ID/icon.png",
  "environment": "*",
  "entrypoints": {
    "main": ["$PACKAGE.$CLASS_NAME"]
  },
  "mixins": ["$MOD_ID.mixins.json"],
  "depends": {
    "fabricloader": ">=0.16.9",
    "minecraft": "~1.21.4",
    "java": ">=21",
    "fabric-api": "*"
  }
}
EOF

# 5. Move and rename Java source
echo -e "${GRAY}  Restructuring source directories...${NC}"
mkdir -p "$NEW_PACKAGE_PATH/mixin"

# Create main mod class
cat > "$NEW_PACKAGE_PATH/$CLASS_NAME.java" << EOF
package $PACKAGE;

import net.fabricmc.api.ModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class $CLASS_NAME implements ModInitializer {
    public static final String MOD_ID = "$MOD_ID";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    @Override
    public void onInitialize() {
        LOGGER.info("Initializing $MOD_NAME");
    }
}
EOF

# Create mixin package-info
cat > "$NEW_PACKAGE_PATH/mixin/package-info.java" << EOF
/** Mixin classes for $MOD_NAME */
package $PACKAGE.mixin;
EOF

# Remove old source structure (only if different from new path)
if [ -d "$OLD_PACKAGE_PATH" ] && [ "$OLD_PACKAGE_PATH" != "$NEW_PACKAGE_PATH" ]; then
    rm -rf "$OLD_PACKAGE_PATH"
    # Clean up empty parent directories
    parent_path="$(dirname "$OLD_PACKAGE_PATH")"
    while [ "$parent_path" != "$JAVA_DIR" ]; do
        if [ -z "$(ls -A "$parent_path" 2>/dev/null)" ]; then
            rmdir "$parent_path"
            parent_path="$(dirname "$parent_path")"
        else
            break
        fi
    done
fi

# 6. Move assets directory
echo -e "${GRAY}  Renaming assets directory...${NC}"
if [ -d "$OLD_ASSETS_DIR" ] && [ "$OLD_ASSETS_DIR" != "$NEW_ASSETS_DIR" ]; then
    mv "$OLD_ASSETS_DIR" "$NEW_ASSETS_DIR"
fi

# 7. Update LICENSE copyright
echo -e "${GRAY}  Updating LICENSE...${NC}"
YEAR=$(date +%Y)
sed -i.bak "s/Copyright (c) [0-9]* Your Name/Copyright (c) $YEAR $AUTHOR/" "$SCRIPT_DIR/LICENSE"
rm -f "$SCRIPT_DIR/LICENSE.bak"

echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Replace assets/$MOD_ID/icon.png.txt with your mod icon (128x128 PNG)"
echo "  2. Run './gradlew build' to verify the setup"
echo "  3. Open in your IDE (IntelliJ IDEA recommended)"
echo "  4. Start coding your mod in src/main/java/$(to_package_path "$PACKAGE")/"
echo ""
echo -e "${GRAY}Optional: Delete this setup script after use${NC}"
