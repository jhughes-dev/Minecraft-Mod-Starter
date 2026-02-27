# Local test runner - mirrors the GitHub Actions CI workflow.
# Builds mcmod CLI and tests scaffolding for both java and kotlin, then diffs against golden files.
# Usage: .\test\run-tests.ps1 [-Language java|kotlin]  (omit -Language to run both)

param(
    [ValidateSet("java", "kotlin", "all")]
    [string]$Language = "all",
    [switch]$TestCI
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

# Build CLI
Write-Host "`nBuilding mcmod CLI..." -ForegroundColor Cyan
Push-Location (Join-Path $RepoRoot "cli")
try {
    cargo build
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
} finally {
    Pop-Location
}

$McmodBin = Join-Path $RepoRoot "cli/target/debug/mcmod.exe"
if (-not (Test-Path $McmodBin)) {
    throw "mcmod binary not found at $McmodBin"
}
Write-Host "  CLI built: $McmodBin" -ForegroundColor Green

function Compare-Golden($actual, $expected) {
    if (-not (Test-Path $actual)) {
        Write-Host "  MISSING: $actual" -ForegroundColor Red
        return $false
    }
    if (-not (Test-Path $expected)) {
        Write-Host "  MISSING GOLDEN: $expected" -ForegroundColor Red
        return $false
    }
    $a = (Get-Content $actual -Raw) -replace "`r`n", "`n"
    $e = (Get-Content $expected -Raw) -replace "`r`n", "`n"
    if ($a -ne $e) {
        Write-Host "  MISMATCH: $actual" -ForegroundColor Red
        $aLines = $a -split "`n"
        $eLines = $e -split "`n"
        $max = [Math]::Max($aLines.Count, $eLines.Count)
        for ($i = 0; $i -lt $max; $i++) {
            $el = if ($i -lt $eLines.Count) { $eLines[$i] } else { "" }
            $al = if ($i -lt $aLines.Count) { $aLines[$i] } else { "" }
            if ($el -ne $al) {
                Write-Host "    Line $($i+1):" -ForegroundColor Yellow
                Write-Host "      - $el" -ForegroundColor Red
                Write-Host "      + $al" -ForegroundColor Green
            }
        }
        return $false
    }
    Write-Host "  OK: $actual" -ForegroundColor DarkGray
    return $true
}

function Run-TestForLanguage($lang) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  Testing: mcmod init + $lang$(if ($TestCI) { ' + CI' } else { '' })" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    # Create empty temp dir
    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "mc-mod-test-$lang-$(Get-Random)"
    Write-Host "  Temp dir: $tempDir" -ForegroundColor Gray

    try {
        $null = New-Item -ItemType Directory -Path $tempDir -Force

        # Run mcmod init
        Write-Host "  Running mcmod init..." -ForegroundColor Gray
        $ciFlag = if ($TestCI) { "true" } else { "false" }
        & $McmodBin init --dir $tempDir `
            --mod-id testmod --mod-name "Test Mod" `
            --package com.example.testmod --author TestAuthor `
            --description "A test mod" --language $lang `
            --loader fabric --loader neoforge `
            --ci $ciFlag --server false --publishing false `
            --testing false --offline
        if ($LASTEXITCODE -ne 0) { throw "mcmod init failed" }

        $fail = $false

        # --- Verify gradle.properties ---
        Write-Host "`n  Checking gradle.properties..." -ForegroundColor Yellow
        $props = Get-Content "$tempDir\gradle.properties" -Raw
        @('archives_base_name=testmod', 'maven_group=com.example.testmod', 'mod_name=Test Mod') | ForEach-Object {
            if ($props -notmatch [regex]::Escape($_)) {
                Write-Host "  FAIL: gradle.properties missing '$_'" -ForegroundColor Red
                $fail = $true
            }
        }

        # --- Verify Fabric API enabled ---
        Write-Host "  Checking Fabric API enabled..." -ForegroundColor Yellow
        $fbg = Get-Content "$tempDir\fabric\build.gradle" -Raw
        if ($fbg -notmatch 'modApi "net.fabricmc.fabric-api:fabric-api') {
            Write-Host "  FAIL: Fabric API not enabled" -ForegroundColor Red
            $fail = $true
        }

        # --- Verify Kotlin-specific ---
        if ($lang -eq "kotlin") {
            Write-Host "  Checking Kotlin-specific settings..." -ForegroundColor Yellow
            if ($props -notmatch '(?m)^mod_language=kotlin') {
                Write-Host "  FAIL: mod_language=kotlin not set" -ForegroundColor Red
                $fail = $true
            }
            if ($props -notmatch '(?m)^kotlin_version=') {
                Write-Host "  FAIL: kotlin_version not set" -ForegroundColor Red
                $fail = $true
            }
            if (Test-Path "$tempDir\common\src\main\java\com\example\testmod\TestmodMod.java") {
                Write-Host "  FAIL: Java source should not exist for kotlin" -ForegroundColor Red
                $fail = $true
            }
        }

        # --- Verify assets ---
        Write-Host "  Checking assets..." -ForegroundColor Yellow
        if (-not (Test-Path "$tempDir\common\src\main\resources\assets\testmod")) {
            Write-Host "  FAIL: Assets directory not found" -ForegroundColor Red
            $fail = $true
        }

        # --- Verify CI setup ---
        Write-Host "  Checking CI setup..." -ForegroundColor Yellow
        if ($TestCI) {
            if (-not (Test-Path "$tempDir\.github\workflows\build.yml")) {
                Write-Host "  FAIL: build.yml not found (CI enabled)" -ForegroundColor Red
                $fail = $true
            }
        } else {
            if (Test-Path "$tempDir\.github") {
                Write-Host "  FAIL: .github should not exist (CI disabled)" -ForegroundColor Red
                $fail = $true
            }
        }

        # --- Normalize versions ---
        Write-Host "  Normalizing versions..." -ForegroundColor Yellow
        & "$RepoRoot\test\normalize-versions.ps1" -File "$tempDir\fabric\src\main\resources\fabric.mod.json"
        & "$RepoRoot\test\normalize-versions.ps1" -File "$tempDir\neoforge\src\main\resources\META-INF\neoforge.mods.toml"

        # --- Golden file diffs ---
        Write-Host "`n  Diffing against golden files..." -ForegroundColor Yellow
        $golden = Join-Path $RepoRoot "test\golden\$lang"

        if ($lang -eq "java") {
            if (-not (Compare-Golden "$tempDir\common\src\main\java\com\example\testmod\TestmodMod.java" `
                "$golden\common\src\main\java\com\example\testmod\TestmodMod.java")) { $fail = $true }
            if (-not (Compare-Golden "$tempDir\fabric\src\main\java\com\example\testmod\fabric\TestmodModFabric.java" `
                "$golden\fabric\src\main\java\com\example\testmod\fabric\TestmodModFabric.java")) { $fail = $true }
            if (-not (Compare-Golden "$tempDir\fabric\src\main\java\com\example\testmod\mixin\package-info.java" `
                "$golden\fabric\src\main\java\com\example\testmod\mixin\package-info.java")) { $fail = $true }
            if (-not (Compare-Golden "$tempDir\neoforge\src\main\java\com\example\testmod\neoforge\TestmodModNeoForge.java" `
                "$golden\neoforge\src\main\java\com\example\testmod\neoforge\TestmodModNeoForge.java")) { $fail = $true }
        } else {
            if (-not (Compare-Golden "$tempDir\common\src\main\kotlin\com\example\testmod\TestmodMod.kt" `
                "$golden\common\src\main\kotlin\com\example\testmod\TestmodMod.kt")) { $fail = $true }
            if (-not (Compare-Golden "$tempDir\fabric\src\main\kotlin\com\example\testmod\fabric\TestmodModFabric.kt" `
                "$golden\fabric\src\main\kotlin\com\example\testmod\fabric\TestmodModFabric.kt")) { $fail = $true }
            if (-not (Compare-Golden "$tempDir\fabric\src\main\java\com\example\testmod\mixin\package-info.java" `
                "$golden\fabric\src\main\java\com\example\testmod\mixin\package-info.java")) { $fail = $true }
            if (-not (Compare-Golden "$tempDir\neoforge\src\main\kotlin\com\example\testmod\neoforge\TestmodModNeoForge.kt" `
                "$golden\neoforge\src\main\kotlin\com\example\testmod\neoforge\TestmodModNeoForge.kt")) { $fail = $true }
        }

        if (-not (Compare-Golden "$tempDir\fabric\src\main\resources\fabric.mod.json" `
            "$golden\fabric\src\main\resources\fabric.mod.json")) { $fail = $true }
        if (-not (Compare-Golden "$tempDir\fabric\src\main\resources\testmod.mixins.json" `
            "$golden\fabric\src\main\resources\testmod.mixins.json")) { $fail = $true }
        if (-not (Compare-Golden "$tempDir\neoforge\src\main\resources\META-INF\neoforge.mods.toml" `
            "$golden\neoforge\src\main\resources\META-INF\neoforge.mods.toml")) { $fail = $true }
        if (-not (Compare-Golden "$tempDir\settings.gradle" `
            "$golden\settings.gradle")) { $fail = $true }

        # --- CI-specific golden file: build.yml ---
        if ($TestCI) {
            Write-Host "`n  Diffing CI-specific golden files..." -ForegroundColor Yellow
            $ciGolden = Join-Path $RepoRoot "test\golden\ci"
            if (-not (Compare-Golden "$tempDir\.github\workflows\build.yml" `
                "$ciGolden\build.yml")) { $fail = $true }
        }

        if ($fail) {
            Write-Host "`n  FAILED: $lang" -ForegroundColor Red
            return $false
        } else {
            Write-Host "`n  PASSED: $lang" -ForegroundColor Green
            return $true
        }
    } finally {
        # Cleanup
        if (Test-Path $tempDir) {
            Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
        }
    }
}

# --- Main ---
$languages = if ($Language -eq "all") { @("java", "kotlin") } else { @($Language) }
$results = @{}

foreach ($lang in $languages) {
    $results[$lang] = Run-TestForLanguage $lang
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Results" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
foreach ($lang in $results.Keys) {
    $status = if ($results[$lang]) { "PASS" } else { "FAIL" }
    $color = if ($results[$lang]) { "Green" } else { "Red" }
    Write-Host "  $lang : $status" -ForegroundColor $color
}

$anyFailed = $results.Values | Where-Object { $_ -eq $false }
if ($anyFailed) {
    Write-Host "`nSome tests failed." -ForegroundColor Red
    exit 1
} else {
    Write-Host "`nAll tests passed." -ForegroundColor Green
    exit 0
}
