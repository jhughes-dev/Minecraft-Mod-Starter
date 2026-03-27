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
            --minecraft 1.21.1 `
            --ci $ciFlag --server false --publishing false `
            --testing false
        if ($LASTEXITCODE -ne 0) { throw "mcmod init failed" }

        $fail = $false

        # --- Verify gradle.properties ---
        Write-Host "`n  Checking gradle.properties..." -ForegroundColor Yellow
        $props = Get-Content "$tempDir\gradle.properties" -Raw
        @('mod.id=testmod', 'mod.group=com.example.testmod', 'mod.name=Test Mod') | ForEach-Object {
            if ($props -notmatch [regex]::Escape($_)) {
                Write-Host "  FAIL: gradle.properties missing '$_'" -ForegroundColor Red
                $fail = $true
            }
        }

        # --- Verify Stonecraft build.gradle.kts ---
        Write-Host "  Checking build.gradle.kts..." -ForegroundColor Yellow
        $bgk = Get-Content "$tempDir\build.gradle.kts" -Raw
        if ($bgk -notmatch 'gg\.meza\.stonecraft') {
            Write-Host "  FAIL: Stonecraft plugin not found in build.gradle.kts" -ForegroundColor Red
            $fail = $true
        }

        # --- Verify settings uses Stonecutter 0.8 ---
        Write-Host "  Checking settings.gradle.kts..." -ForegroundColor Yellow
        $settings = Get-Content "$tempDir\settings.gradle.kts" -Raw
        if ($settings -notmatch 'stonecutter.*0\.8') {
            Write-Host "  FAIL: Stonecutter 0.8 not found in settings.gradle.kts" -ForegroundColor Red
            $fail = $true
        }

        # --- Verify unified source (no fabric/ or neoforge/ dirs) ---
        Write-Host "  Checking unified source layout..." -ForegroundColor Yellow
        if (Test-Path "$tempDir\fabric") {
            Write-Host "  FAIL: fabric/ directory should not exist" -ForegroundColor Red
            $fail = $true
        }
        if (Test-Path "$tempDir\neoforge") {
            Write-Host "  FAIL: neoforge/ directory should not exist" -ForegroundColor Red
            $fail = $true
        }

        # --- Verify assets ---
        Write-Host "  Checking assets..." -ForegroundColor Yellow
        if (-not (Test-Path "$tempDir\src\main\resources\assets\testmod")) {
            Write-Host "  FAIL: Assets directory not found" -ForegroundColor Red
            $fail = $true
        }

        # --- Verify version properties ---
        Write-Host "  Checking version properties..." -ForegroundColor Yellow
        if (-not (Test-Path "$tempDir\versions\dependencies\1.21.1.properties")) {
            Write-Host "  FAIL: Version properties not found" -ForegroundColor Red
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

        # --- Golden file diffs ---
        Write-Host "`n  Diffing against golden files..." -ForegroundColor Yellow
        $golden = Join-Path $RepoRoot "test\golden\$lang"

        $goldenPairs = @()
        if ($lang -eq "java") {
            $goldenPairs += @(
                @("src\main\java\com\example\testmod\TestmodMod.java"),
                @("src\main\java\com\example\testmod\mixin\package-info.java")
            )
        } else {
            $goldenPairs += @(
                @("src\main\kotlin\com\example\testmod\TestmodMod.kt"),
                @("src\main\java\com\example\testmod\mixin\package-info.java")
            )
        }
        $goldenPairs += @(
            @("src\main\resources\fabric.mod.json"),
            @("src\main\resources\testmod.mixins.json"),
            @("src\main\resources\META-INF\neoforge.mods.toml"),
            @("settings.gradle.kts")
        )

        foreach ($relPath in $goldenPairs) {
            $r = Compare-Golden "$tempDir\$relPath" "$golden\$relPath"
            if ($r -eq $false) { $fail = $true }
        }

        # --- CI-specific golden file: build.yml ---
        if ($TestCI) {
            Write-Host "`n  Diffing CI-specific golden files..." -ForegroundColor Yellow
            $ciGolden = Join-Path $RepoRoot "test\golden\ci"
            $r = Compare-Golden "$tempDir\.github\workflows\build.yml" "$ciGolden\build.yml"
            if ($r -eq $false) { $fail = $true }
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
