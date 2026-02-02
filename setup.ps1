# Multi-Loader Mod Setup Script
# Customizes this boilerplate for a new mod project (Fabric + NeoForge via Architectury)

param(
    [string]$ModId,
    [string]$ModName,
    [string]$Package,
    [string]$Author,
    [string]$Description,
    [string]$MinecraftVersion,
    [ValidateSet("java", "kotlin")]
    [string]$Language,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

function Write-Header {
    Write-Host ""
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host "  Multi-Loader Mod Setup" -ForegroundColor Cyan
    Write-Host "  (Fabric + NeoForge)" -ForegroundColor Cyan
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

function ConvertTo-JsonSafeString {
    param([string]$Value)
    return $Value -replace '\\', '\\' -replace '"', '\"' -replace "`n", '\n' -replace "`r", '' -replace "`t", '\t'
}

function ConvertTo-TomlSafeString {
    param([string]$Value)
    return $Value -replace '\\', '\\' -replace '"', '\"' -replace "`n", '\n' -replace "`r", '' -replace "`t", '\t'
}

# --- Version Auto-Detection ---

function Get-LatestMinecraftVersion {
    Write-Host "  Querying latest stable Minecraft version..." -ForegroundColor Gray
    try {
        $versions = Invoke-RestMethod -Uri "https://meta.fabricmc.net/v2/versions/game" -TimeoutSec 10
        $stable = $versions | Where-Object { $_.stable -eq $true } | Select-Object -First 1
        return $stable.version
    } catch {
        Write-Host "  Warning: Could not fetch Minecraft versions. Using default." -ForegroundColor Yellow
        return $null
    }
}

function Get-LatestLoaderVersion {
    Write-Host "  Querying latest stable Fabric Loader version..." -ForegroundColor Gray
    try {
        $versions = Invoke-RestMethod -Uri "https://meta.fabricmc.net/v2/versions/loader" -TimeoutSec 10
        $stable = $versions | Where-Object { $_.stable -eq $true } | Select-Object -First 1
        return $stable.version
    } catch {
        Write-Host "  Warning: Could not fetch loader versions. Using default." -ForegroundColor Yellow
        return $null
    }
}

function Get-LatestFabricApiVersion {
    param([string]$McVersion)
    Write-Host "  Querying latest Fabric API for Minecraft $McVersion..." -ForegroundColor Gray
    try {
        $xml = [xml](Invoke-WebRequest -Uri "https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/maven-metadata.xml" -TimeoutSec 10 -UseBasicParsing).Content
        $allVersions = $xml.metadata.versioning.versions.version
        $matching = $allVersions | Where-Object { $_ -like "*+$McVersion" } | Select-Object -Last 1
        if ($matching) { return $matching }
        Write-Host "  Warning: No Fabric API found for MC $McVersion. Trying without patch version..." -ForegroundColor Yellow
        $majorMinor = ($McVersion -split '\.')[0..1] -join '.'
        $matching = $allVersions | Where-Object { $_ -like "*+$majorMinor" } | Select-Object -Last 1
        return $matching
    } catch {
        Write-Host "  Warning: Could not fetch Fabric API versions. Using default." -ForegroundColor Yellow
        return $null
    }
}

function Get-LatestNeoForgeVersion {
    param([string]$McVersion)
    Write-Host "  Querying latest NeoForge for Minecraft $McVersion..." -ForegroundColor Gray
    try {
        $xml = [xml](Invoke-WebRequest -Uri "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml" -TimeoutSec 10 -UseBasicParsing).Content
        $allVersions = $xml.metadata.versioning.versions.version
        # MC 1.X.Y -> NeoForge prefix X.Y
        $parts = $McVersion -split '\.'
        $neoPrefix = "$($parts[1]).$($parts[2])"
        $matching = $allVersions | Where-Object { $_ -like "$neoPrefix.*" } | Select-Object -Last 1
        return $matching
    } catch {
        Write-Host "  Warning: Could not fetch NeoForge versions. Using default." -ForegroundColor Yellow
        return $null
    }
}

# --- Main Script ---

Write-Header

# Gather mod info if not provided as parameters
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
    $Description = Get-Input -Prompt "Mod description" -Default "A Minecraft mod"
}
if ([string]::IsNullOrWhiteSpace($Language)) {
    $Language = Get-Input -Prompt "Language (java/kotlin)" -Default "java"
}
$Language = $Language.ToLower()
if ($Language -ne "java" -and $Language -ne "kotlin") {
    Write-Host "Error: Language must be 'java' or 'kotlin'" -ForegroundColor Red
    exit 1
}
$UseKotlin = $Language -eq "kotlin"

# Validate mod ID
if ($ModId -notmatch '^[a-z][a-z0-9_]*$') {
    Write-Host "Error: Mod ID must be lowercase, start with a letter, and contain only a-z, 0-9, _" -ForegroundColor Red
    exit 1
}

# Fetch latest versions
Write-Host "Fetching latest versions..." -ForegroundColor Cyan

# Read current defaults from gradle.properties
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$propsPath = Join-Path $scriptDir "gradle.properties"
$currentProps = Get-Content $propsPath -Raw
$defaultMcVersion = if ($currentProps -match 'minecraft_version=(.+)') { $Matches[1].Trim() } else { "1.21.4" }
$defaultLoaderVersion = if ($currentProps -match 'fabric_loader_version=(.+)') { $Matches[1].Trim() } else { "0.16.9" }
$defaultFabricVersion = if ($currentProps -match '(?:#\s*)?fabric_api_version=(.+)') { $Matches[1].Trim() } else { "0.111.0+1.21.4" }
$defaultNeoForgeVersion = if ($currentProps -match 'neoforge_version=(.+)') { $Matches[1].Trim() } else { "21.4.156" }

$fetchedMcVersion = Get-LatestMinecraftVersion
$fetchedLoaderVersion = Get-LatestLoaderVersion

# Use fetched MC version or parameter or default
$mcVersion = if (-not [string]::IsNullOrWhiteSpace($MinecraftVersion)) { $MinecraftVersion }
             elseif ($fetchedMcVersion) { $fetchedMcVersion }
             else { $defaultMcVersion }

$loaderVersion = if ($fetchedLoaderVersion) { $fetchedLoaderVersion } else { $defaultLoaderVersion }

$fetchedFabricVersion = Get-LatestFabricApiVersion -McVersion $mcVersion
$fabricVersion = if ($fetchedFabricVersion) { $fetchedFabricVersion } else { $defaultFabricVersion }

$fetchedNeoForgeVersion = Get-LatestNeoForgeVersion -McVersion $mcVersion
$neoForgeVersion = if ($fetchedNeoForgeVersion) { $fetchedNeoForgeVersion } else { $defaultNeoForgeVersion }

# Derive NeoForge major version for dependency range (e.g., 21.4)
$neoForgeMajor = ($neoForgeVersion -split '\.')[0..1] -join '.'

Write-Host ""
Write-Host "Configuration:" -ForegroundColor Yellow
Write-Host "  Mod ID:      $ModId"
Write-Host "  Mod Name:    $ModName"
Write-Host "  Package:     $Package"
Write-Host "  Author:      $Author"
Write-Host "  Description: $Description"
Write-Host "  Language:    $Language"
Write-Host ""
Write-Host "Versions:" -ForegroundColor Yellow
Write-Host "  Minecraft:   $mcVersion"
Write-Host "  Fabric Loader: $loaderVersion"
Write-Host "  Fabric API:  $fabricVersion"
Write-Host "  NeoForge:    $neoForgeVersion"
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

# Derive class name from mod ID (PascalCase + Mod suffix)
$ClassName = ($ModId -split '_' | ForEach-Object { $_.Substring(0,1).ToUpper() + $_.Substring(1) }) -join ''
$ClassNameMod = $ClassName + "Mod"

$packagePath = Convert-ToPackagePath $Package

# --- Paths ---
$srcLang = if ($UseKotlin) { "kotlin" } else { "java" }
$commonSrcDir = Join-Path $scriptDir "common/src/main/$srcLang"
$commonResourcesDir = Join-Path $scriptDir "common/src/main/resources"
$fabricSrcDir = Join-Path $scriptDir "fabric/src/main/$srcLang"
$fabricResourcesDir = Join-Path $scriptDir "fabric/src/main/resources"
$neoforgeSrcDir = Join-Path $scriptDir "neoforge/src/main/$srcLang"
$neoforgeResourcesDir = Join-Path $scriptDir "neoforge/src/main/resources"

# Old paths (always in java dir, from template defaults)
$commonJavaDir = Join-Path $scriptDir "common/src/main/java"
$fabricJavaDir = Join-Path $scriptDir "fabric/src/main/java"
$neoforgeJavaDir = Join-Path $scriptDir "neoforge/src/main/java"

$oldCommonPackagePath = Join-Path $commonJavaDir "io/github/yourname/modid"
$newCommonPackagePath = Join-Path $commonSrcDir $packagePath
$oldFabricPackagePath = Join-Path $fabricJavaDir "io/github/yourname/modid/fabric"
$newFabricPackagePath = Join-Path $fabricSrcDir "$packagePath/fabric"
$oldNeoforgePackagePath = Join-Path $neoforgeJavaDir "io/github/yourname/modid/neoforge"
$newNeoforgePackagePath = Join-Path $neoforgeSrcDir "$packagePath/neoforge"

# 1. Update gradle.properties
Write-Host "  Updating gradle.properties..." -ForegroundColor Gray
$gradleProps = Get-Content (Join-Path $scriptDir "gradle.properties") -Raw
$gradleProps = $gradleProps -replace 'minecraft_version=.*', "minecraft_version=$mcVersion"
$gradleProps = $gradleProps -replace 'fabric_loader_version=.*', "fabric_loader_version=$loaderVersion"
$gradleProps = $gradleProps -replace '# fabric_api_version=.*', "fabric_api_version=$fabricVersion"
$gradleProps = $gradleProps -replace 'neoforge_version=.*', "neoforge_version=$neoForgeVersion"
$gradleProps = $gradleProps -replace '# mod_language=.*', "mod_language=$Language"
if ($UseKotlin) {
    $gradleProps = $gradleProps -replace '# kotlin_version=(.*)', 'kotlin_version=`$1'
}
$gradleProps = $gradleProps -replace 'maven_group=.*', "maven_group=$Package"
$gradleProps = $gradleProps -replace 'archives_base_name=.*', "archives_base_name=$ModId"
$gradleProps = $gradleProps -replace 'mod_name=.*', "mod_name=$ModName"
Set-Content (Join-Path $scriptDir "gradle.properties") $gradleProps -NoNewline

# 3. Update settings.gradle - set rootProject.name
Write-Host "  Updating settings.gradle..." -ForegroundColor Gray
$settingsGradle = Get-Content (Join-Path $scriptDir "settings.gradle") -Raw
$settingsGradle = $settingsGradle -replace 'rootProject\.name = "modid"', "rootProject.name = `"$ModId`""
Set-Content (Join-Path $scriptDir "settings.gradle") $settingsGradle -NoNewline

# 4. Enable Fabric API in fabric/build.gradle
Write-Host "  Enabling Fabric API in fabric/build.gradle..." -ForegroundColor Gray
$fabricBuildGradle = Get-Content (Join-Path $scriptDir "fabric/build.gradle") -Raw
$fabricBuildGradle = $fabricBuildGradle -replace '// modApi "net.fabricmc.fabric-api:fabric-api', 'modApi "net.fabricmc.fabric-api:fabric-api'
Set-Content (Join-Path $scriptDir "fabric/build.gradle") $fabricBuildGradle -NoNewline

# 5. Create common module source
Write-Host "  Creating common module source..." -ForegroundColor Gray
$null = New-Item -ItemType Directory -Path $newCommonPackagePath -Force

if ($UseKotlin) {
    $commonClass = @"
package $Package

import org.slf4j.LoggerFactory

object $ClassNameMod {
    const val MOD_ID = "$ModId"
    val LOGGER = LoggerFactory.getLogger(MOD_ID)

    fun init() {
        LOGGER.info("Initializing $ModName")
    }
}
"@
    Set-Content (Join-Path $newCommonPackagePath "$ClassNameMod.kt") $commonClass
} else {
    $commonClass = @"
package $Package;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class $ClassNameMod {
    public static final String MOD_ID = "$ModId";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    public static void init() {
        LOGGER.info("Initializing $ModName");
    }
}
"@
    Set-Content (Join-Path $newCommonPackagePath "$ClassNameMod.java") $commonClass
}

# Remove old common source
if ((Test-Path $oldCommonPackagePath) -and ($oldCommonPackagePath -ne $newCommonPackagePath)) {
    Remove-Item -Recurse -Force $oldCommonPackagePath
    $parentPath = Split-Path $oldCommonPackagePath -Parent
    while ($parentPath -ne $commonJavaDir) {
        if ((Get-ChildItem $parentPath -Force -ErrorAction SilentlyContinue | Measure-Object).Count -eq 0) {
            Remove-Item $parentPath -Force
            $parentPath = Split-Path $parentPath -Parent
        } else { break }
    }
}
# If using Kotlin, also remove the default java source dir if empty
if ($UseKotlin -and (Test-Path $commonJavaDir)) {
    $remaining = Get-ChildItem $commonJavaDir -Recurse -File -ErrorAction SilentlyContinue
    if (-not $remaining) { Remove-Item -Recurse -Force $commonJavaDir }
}

# Move common assets
$oldCommonAssets = Join-Path $commonResourcesDir "assets/modid"
$newCommonAssets = Join-Path $commonResourcesDir "assets/$ModId"
if ((Test-Path $oldCommonAssets) -and ($oldCommonAssets -ne $newCommonAssets)) {
    Move-Item $oldCommonAssets $newCommonAssets -Force
}

# 6. Create Fabric module source
Write-Host "  Creating Fabric module source..." -ForegroundColor Gray
$null = New-Item -ItemType Directory -Path $newFabricPackagePath -Force
$null = New-Item -ItemType Directory -Path (Join-Path (Split-Path $newFabricPackagePath -Parent) "mixin") -Force

if ($UseKotlin) {
    $fabricClass = @"
package $Package.fabric

import $Package.$ClassNameMod
import net.fabricmc.api.ModInitializer

class ${ClassNameMod}Fabric : ModInitializer {
    override fun onInitialize() {
        ${ClassNameMod}.init()
    }
}
"@
    Set-Content (Join-Path $newFabricPackagePath "${ClassNameMod}Fabric.kt") $fabricClass
} else {
    $fabricClass = @"
package $Package.fabric;

import $Package.$ClassNameMod;
import net.fabricmc.api.ModInitializer;

public class ${ClassNameMod}Fabric implements ModInitializer {
    @Override
    public void onInitialize() {
        ${ClassNameMod}.init();
    }
}
"@
    Set-Content (Join-Path $newFabricPackagePath "${ClassNameMod}Fabric.java") $fabricClass
}

# Create mixin package-info for fabric
Set-Content (Join-Path (Split-Path $newFabricPackagePath -Parent) "mixin/package-info.java") "/** Mixin classes for $ModName */`npackage $Package.mixin;"

# Remove old fabric source
if ((Test-Path $oldFabricPackagePath) -and ($oldFabricPackagePath -ne $newFabricPackagePath)) {
    Remove-Item -Recurse -Force $oldFabricPackagePath
    $parentPath = Split-Path $oldFabricPackagePath -Parent
    while ($parentPath -ne $fabricJavaDir) {
        if ((Get-ChildItem $parentPath -Force -ErrorAction SilentlyContinue | Measure-Object).Count -eq 0) {
            Remove-Item $parentPath -Force
            $parentPath = Split-Path $parentPath -Parent
        } else { break }
    }
}
# If using Kotlin, clean up default java source dir (keep mixin dir which stays in java)
if ($UseKotlin -and (Test-Path $fabricJavaDir)) {
    # Remove everything except mixin-related files
    $oldJavaPackage = Join-Path $fabricJavaDir "io/github/yourname/modid"
    if (Test-Path $oldJavaPackage) { Remove-Item -Recurse -Force $oldJavaPackage }
}

# Escape user input for JSON/TOML
$SafeDescription = ConvertTo-JsonSafeString $Description
$SafeAuthor = ConvertTo-JsonSafeString $Author
$SafeModName = ConvertTo-JsonSafeString $ModName
$SafeDescriptionToml = ConvertTo-TomlSafeString $Description
$SafeAuthorToml = ConvertTo-TomlSafeString $Author
$SafeModNameToml = ConvertTo-TomlSafeString $ModName

# Create fabric.mod.json
$fabricJson = @"
{
  "schemaVersion": 1,
  "id": "$ModId",
  "version": "`${version}",
  "name": "$SafeModName",
  "description": "$SafeDescription",
  "authors": ["$SafeAuthor"],
  "contact": {
    "homepage": "https://github.com/yourname/$ModId",
    "sources": "https://github.com/yourname/$ModId"
  },
  "license": "MIT",
  "icon": "assets/$ModId/icon.png",
  "environment": "*",
  "entrypoints": {
    "main": ["$Package.fabric.${ClassNameMod}Fabric"]
  },
  "mixins": ["$ModId.mixins.json"],
  "depends": {
    "fabricloader": ">=$loaderVersion",
    "minecraft": "~$mcVersion",
    "java": ">=21",
    "fabric-api": "*"
  }
}
"@
Set-Content (Join-Path $fabricResourcesDir "fabric.mod.json") $fabricJson

# Create mixin config for fabric
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
Set-Content (Join-Path $fabricResourcesDir "$ModId.mixins.json") $mixinConfig

# 7. Create NeoForge module source
Write-Host "  Creating NeoForge module source..." -ForegroundColor Gray
$null = New-Item -ItemType Directory -Path $newNeoforgePackagePath -Force

if ($UseKotlin) {
    $neoforgeClass = @"
package $Package.neoforge

import $Package.$ClassNameMod
import net.neoforged.bus.api.IEventBus
import net.neoforged.fml.common.Mod

@Mod(${ClassNameMod}.MOD_ID)
class ${ClassNameMod}NeoForge(modEventBus: IEventBus) {
    init {
        ${ClassNameMod}.init()
    }
}
"@
    Set-Content (Join-Path $newNeoforgePackagePath "${ClassNameMod}NeoForge.kt") $neoforgeClass
} else {
    $neoforgeClass = @"
package $Package.neoforge;

import $Package.$ClassNameMod;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;

@Mod(${ClassNameMod}.MOD_ID)
public class ${ClassNameMod}NeoForge {
    public ${ClassNameMod}NeoForge(IEventBus modEventBus) {
        ${ClassNameMod}.init();
    }
}
"@
    Set-Content (Join-Path $newNeoforgePackagePath "${ClassNameMod}NeoForge.java") $neoforgeClass
}

# Remove old neoforge source
if ((Test-Path $oldNeoforgePackagePath) -and ($oldNeoforgePackagePath -ne $newNeoforgePackagePath)) {
    Remove-Item -Recurse -Force $oldNeoforgePackagePath
    $parentPath = Split-Path $oldNeoforgePackagePath -Parent
    while ($parentPath -ne $neoforgeJavaDir) {
        if ((Get-ChildItem $parentPath -Force -ErrorAction SilentlyContinue | Measure-Object).Count -eq 0) {
            Remove-Item $parentPath -Force
            $parentPath = Split-Path $parentPath -Parent
        } else { break }
    }
}
if ($UseKotlin -and (Test-Path $neoforgeJavaDir)) {
    $remaining = Get-ChildItem $neoforgeJavaDir -Recurse -File -ErrorAction SilentlyContinue
    if (-not $remaining) { Remove-Item -Recurse -Force $neoforgeJavaDir }
}

# Create neoforge.mods.toml
$null = New-Item -ItemType Directory -Path (Join-Path $neoforgeResourcesDir "META-INF") -Force
$neoforgeToml = @"
modLoader = "javafml"
loaderVersion = "[4,)"
license = "MIT"

[[mods]]
modId = "$ModId"
version = "`${version}"
displayName = "$SafeModNameToml"
description = "$SafeDescriptionToml"
authors = "$SafeAuthorToml"
logoFile = "assets/$ModId/icon.png"

[[dependencies.$ModId]]
modId = "neoforge"
type = "required"
versionRange = "[$neoForgeMajor,)"
ordering = "NONE"
side = "BOTH"

[[dependencies.$ModId]]
modId = "minecraft"
type = "required"
versionRange = "[$mcVersion,)"
ordering = "NONE"
side = "BOTH"
"@
Set-Content (Join-Path $neoforgeResourcesDir "META-INF/neoforge.mods.toml") $neoforgeToml

# 8. Update LICENSE copyright
Write-Host "  Updating LICENSE..." -ForegroundColor Gray
$license = Get-Content (Join-Path $scriptDir "LICENSE") -Raw
$license = $license -replace 'Copyright \(c\) \d+ Your Name', "Copyright (c) $(Get-Date -Format yyyy) $Author"
Set-Content (Join-Path $scriptDir "LICENSE") $license -NoNewline

Write-Host ""
Write-Host "Setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Replace common/src/main/resources/assets/$ModId/icon.png.txt with your mod icon (128x128 PNG)"
Write-Host "  2. Run './gradlew build' to verify the setup"
Write-Host "  3. Open in your IDE (IntelliJ IDEA recommended)"
Write-Host ""
Write-Host "Project structure:" -ForegroundColor Yellow
Write-Host "  common/ - Shared code (both loaders)"
Write-Host "  fabric/ - Fabric-specific code"
Write-Host "  neoforge/ - NeoForge-specific code"
Write-Host ""
Write-Host "Build outputs:" -ForegroundColor Yellow
Write-Host "  fabric/build/libs/$ModId-*.jar"
Write-Host "  neoforge/build/libs/$ModId-*.jar"
Write-Host ""
Write-Host "Optional: Delete this setup script after use" -ForegroundColor Gray
