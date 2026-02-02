# Local test runner - mirrors the GitHub Actions CI workflow.
# Runs setup.ps1 for both java and kotlin in isolated temp copies, then diffs against golden files.
# Usage: .\test\run-tests.ps1 [-Language java|kotlin]  (omit -Language to run both)

param(
    [ValidateSet("java", "kotlin", "all")]
    [string]$Language = "all"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$TotalFail = $false

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
    Write-Host "  Testing: PowerShell + $lang" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    # Create isolated temp copy
    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "mc-mod-test-$lang-$(Get-Random)"
    Write-Host "  Temp dir: $tempDir" -ForegroundColor Gray

    try {
        # Copy repo to temp (exclude .git, build, test output dirs)
        robocopy $RepoRoot $tempDir /E /NFL /NDL /NJH /NJS /XD ".git" "build" ".gradle" "out" | Out-Null

        # Run setup
        Write-Host "  Running setup.ps1..." -ForegroundColor Gray
        & "$tempDir\setup.ps1" -ModId "testmod" -ModName "Test Mod" -Package "com.example.testmod" `
            -Author "TestAuthor" -Description "A test mod" -Language $lang -Force

        $fail = $false

        # --- Verify gradle.properties ---
        Write-Host "`n  Checking gradle.properties..." -ForegroundColor Yellow
        $props = Get-Content "$tempDir\gradle.properties" -Raw
        @('archives_base_name=testmod', 'maven_group=com.example.testmod', 'mod_name=Test Mod', "mod_language=$lang") | ForEach-Object {
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
            if ($props -notmatch '(?m)^kotlin_version=') {
                Write-Host "  FAIL: kotlin_version not enabled" -ForegroundColor Red
                $fail = $true
            }
            if (Test-Path "$tempDir\common\src\main\java\com\example\testmod\TestmodMod.java") {
                Write-Host "  FAIL: Java source not cleaned up" -ForegroundColor Red
                $fail = $true
            }
        }

        # --- Verify assets ---
        Write-Host "  Checking assets renamed..." -ForegroundColor Yellow
        if (-not (Test-Path "$tempDir\common\src\main\resources\assets\testmod")) {
            Write-Host "  FAIL: Assets not renamed" -ForegroundColor Red
            $fail = $true
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
