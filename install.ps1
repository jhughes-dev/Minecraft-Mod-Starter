$ErrorActionPreference = "Stop"

$Repo = "jhughes-dev/Minecraft-Mod-Starter"
$BinaryName = "mcmod.exe"
$InstallDir = Join-Path $env:LOCALAPPDATA "mcmod"
$Asset = "mcmod-windows-x86_64.exe"

# Fetch latest release tag
Write-Host "Fetching latest release..."
$Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
$Tag = $Release.tag_name

if (-not $Tag) {
    Write-Error "Could not determine latest release"
    exit 1
}

Write-Host "Latest release: $Tag"

$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/$Asset"

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$InstallPath = Join-Path $InstallDir $BinaryName

Write-Host "Downloading $Asset..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile $InstallPath -UseBasicParsing

Write-Host "Installed mcmod to $InstallPath"

# Add to user PATH if not already present
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host ""
    Write-Host "Added $InstallDir to your user PATH."
    Write-Host "Restart your terminal, then run 'mcmod --help' to get started."
} else {
    Write-Host "Run 'mcmod --help' to get started."
}
