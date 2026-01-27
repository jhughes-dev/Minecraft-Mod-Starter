# Fabric Mod Setup Script
# Customizes this boilerplate for a new mod project

param(
    [string]$ModId,
    [string]$ModName,
    [string]$Package,
    [string]$Author,
    [string]$Description,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

function Write-Header {
    Write-Host ""
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host "  Fabric Mod Setup Script" -ForegroundColor Cyan
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host ""
}

function Get-Input {
    param([string]$Prompt, [string]$Default)
    $userInput = Read-Host "$Prompt [$Default]"
    if ([string]::IsNullOrWhiteSpace($userInput)) { return $Default }
    return $userInput
}

function Convert-ToPackagePath {
    param([string]$Package)
    return $Package -replace '\.', '/'
}

Write-Header

# Gather input if not provided as parameters
if ([string]::IsNullOrWhiteSpace($ModId)) {
    $ModId = Get-Input -Prompt "Mod ID (lowercase, no spaces, e.g., 'mymod')" -Default "mymod"
}
if ([string]::IsNullOrWhiteSpace($ModName)) {
    $ModName = Get-Input -Prompt "Mod Display Name" -Default "My Mod"
}
if ([string]::IsNullOrWhiteSpace($Package)) {
    $Package = Get-Input -Prompt "Package name (e.g., 'io.github.username.mymod')" -Default "io.github.yourname.$ModId"
}
if ([string]::IsNullOrWhiteSpace($Author)) {
    $Author = Get-Input -Prompt "Author name" -Default "Your Name"
}
if ([string]::IsNullOrWhiteSpace($Description)) {
    $Description = Get-Input -Prompt "Mod description" -Default "A Fabric mod for Minecraft"
}

# Validate mod ID
if ($ModId -notmatch '^[a-z][a-z0-9_]*$') {
    Write-Host "Error: Mod ID must be lowercase, start with a letter, and contain only a-z, 0-9, _" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Configuration:" -ForegroundColor Yellow
Write-Host "  Mod ID:      $ModId"
Write-Host "  Mod Name:    $ModName"
Write-Host "  Package:     $Package"
Write-Host "  Author:      $Author"
Write-Host "  Description: $Description"
Write-Host ""

if (-not $Force) {
    $confirm = Read-Host "Proceed with setup? (y/N)"
    if ($confirm -ne 'y' -and $confirm -ne 'Y') {
        Write-Host "Setup cancelled." -ForegroundColor Yellow
        exit 0
    }
}

Write-Host ""
Write-Host "Setting up mod..." -ForegroundColor Green

# Paths
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$srcDir = Join-Path $scriptDir "src"
$mainDir = Join-Path $srcDir "main"
$javaDir = Join-Path $mainDir "java"
$resourcesDir = Join-Path $mainDir "resources"
$assetsDir = Join-Path $resourcesDir "assets"

$oldPackagePath = Join-Path $javaDir "io/github/yourname/modid"
$newPackagePath = Join-Path $javaDir (Convert-ToPackagePath $Package)
$oldAssetsDir = Join-Path $assetsDir "modid"
$newAssetsDir = Join-Path $assetsDir $ModId

# Derive class name from mod ID (PascalCase)
$ClassName = ($ModId -split '_' | ForEach-Object { $_.Substring(0,1).ToUpper() + $_.Substring(1) }) -join ''
$ClassName = $ClassName + "Mod"

# 1. Update gradle.properties
Write-Host "  Updating gradle.properties..." -ForegroundColor Gray
$gradleProps = Get-Content (Join-Path $scriptDir "gradle.properties") -Raw
$gradleProps = $gradleProps -replace 'maven_group=.*', "maven_group=$Package"
$gradleProps = $gradleProps -replace 'archives_base_name=.*', "archives_base_name=$ModId"
$gradleProps = $gradleProps -replace '# fabric_version=', 'fabric_version='
Set-Content (Join-Path $scriptDir "gradle.properties") $gradleProps -NoNewline

# 2. Update build.gradle - enable Fabric API
Write-Host "  Enabling Fabric API in build.gradle..." -ForegroundColor Gray
$buildGradle = Get-Content (Join-Path $scriptDir "build.gradle") -Raw
$buildGradle = $buildGradle -replace '// modImplementation "net.fabricmc.fabric-api:fabric-api', 'modImplementation "net.fabricmc.fabric-api:fabric-api'
Set-Content (Join-Path $scriptDir "build.gradle") $buildGradle -NoNewline

# 3. Create mixin config file
Write-Host "  Creating mixin configuration..." -ForegroundColor Gray
$mixinConfig = @"
{
  "required": true,
  "minVersion": "0.8",
  "package": "$Package.mixin",
  "compatibilityLevel": "JAVA_21",
  "mixins": [],
  "client": [],
  "server": [],
  "injectors": {
    "defaultRequire": 1
  }
}
"@
Set-Content (Join-Path $resourcesDir "$ModId.mixins.json") $mixinConfig

# 4. Update fabric.mod.json
Write-Host "  Updating fabric.mod.json..." -ForegroundColor Gray
$fabricJson = @"
{
  "schemaVersion": 1,
  "id": "$ModId",
  "version": "`${version}",
  "name": "$ModName",
  "description": "$Description",
  "authors": ["$Author"],
  "contact": {
    "homepage": "https://github.com/yourname/$ModId",
    "sources": "https://github.com/yourname/$ModId"
  },
  "license": "MIT",
  "icon": "assets/$ModId/icon.png",
  "environment": "*",
  "entrypoints": {
    "main": ["$Package.$ClassName"]
  },
  "mixins": ["$ModId.mixins.json"],
  "depends": {
    "fabricloader": ">=0.16.9",
    "minecraft": "~1.21.4",
    "java": ">=21",
    "fabric-api": "*"
  }
}
"@
Set-Content (Join-Path $resourcesDir "fabric.mod.json") $fabricJson

# 5. Move and rename Java source
Write-Host "  Restructuring source directories..." -ForegroundColor Gray
$null = New-Item -ItemType Directory -Path $newPackagePath -Force
$null = New-Item -ItemType Directory -Path (Join-Path $newPackagePath "mixin") -Force

# Create main mod class
$mainClass = @"
package $Package;

import net.fabricmc.api.ModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class $ClassName implements ModInitializer {
    public static final String MOD_ID = "$ModId";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    @Override
    public void onInitialize() {
        LOGGER.info("Initializing $ModName");
    }
}
"@
Set-Content (Join-Path $newPackagePath "$ClassName.java") $mainClass

# Create mixin package-info
Set-Content (Join-Path $newPackagePath "mixin/package-info.java") "/** Mixin classes for $ModName */`npackage $Package.mixin;"

# Remove old source structure (only if different from new path)
if ((Test-Path $oldPackagePath) -and ($oldPackagePath -ne $newPackagePath)) {
    Remove-Item -Recurse -Force $oldPackagePath
    # Clean up empty parent directories
    $parentPath = Split-Path $oldPackagePath -Parent
    while ($parentPath -ne $javaDir) {
        if ((Get-ChildItem $parentPath -Force | Measure-Object).Count -eq 0) {
            Remove-Item $parentPath -Force
            $parentPath = Split-Path $parentPath -Parent
        } else {
            break
        }
    }
}

# 6. Move assets directory
Write-Host "  Renaming assets directory..." -ForegroundColor Gray
if (Test-Path $oldAssetsDir) {
    if ($oldAssetsDir -ne $newAssetsDir) {
        Move-Item $oldAssetsDir $newAssetsDir -Force
    }
}

# 7. Update LICENSE copyright
Write-Host "  Updating LICENSE..." -ForegroundColor Gray
$license = Get-Content (Join-Path $scriptDir "LICENSE") -Raw
$license = $license -replace 'Copyright \(c\) \d+ Your Name', "Copyright (c) $(Get-Date -Format yyyy) $Author"
Set-Content (Join-Path $scriptDir "LICENSE") $license -NoNewline

Write-Host ""
Write-Host "Setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Replace assets/$ModId/icon.png.txt with your mod icon (128x128 PNG)"
Write-Host "  2. Run './gradlew build' to verify the setup"
Write-Host "  3. Open in your IDE (IntelliJ IDEA recommended)"
Write-Host "  4. Start coding your mod in src/main/java/$(Convert-ToPackagePath $Package)/"
Write-Host ""
Write-Host "Optional: Delete this setup script after use" -ForegroundColor Gray
