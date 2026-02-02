#!/usr/bin/env bash
# Replaces dynamic version numbers in setup output with placeholders for gold file comparison.
# Usage: ./normalize-versions.sh <file>
# Modifies the file in-place.

set -e

FILE="$1"

# Normalize Minecraft version (e.g., 1.21.4)
sed -i -E 's/"fabricloader": ">=[0-9]+\.[0-9]+\.[0-9]+"/"fabricloader": ">=__LOADER_VERSION__"/' "$FILE"
sed -i -E 's/"minecraft": "~[0-9]+\.[0-9]+\.[0-9]+"/"minecraft": "~__MC_VERSION__"/' "$FILE"
sed -i -E 's/versionRange = "\[[0-9]+\.[0-9]+,\)"/versionRange = "[__NEOFORGE_MAJOR__,)"/' "$FILE"
sed -i -E 's/versionRange = "\[[0-9]+\.[0-9]+\.[0-9]+,\)"/versionRange = "[__MC_VERSION__,)"/' "$FILE"
