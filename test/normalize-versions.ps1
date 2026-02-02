# Replaces dynamic version numbers in setup output with placeholders for gold file comparison.
# Usage: .\normalize-versions.ps1 <file>
# Modifies the file in-place.

param([string]$File)

$content = Get-Content $File -Raw
$content = $content -replace '"fabricloader": ">=[\d.]+"', '"fabricloader": ">=__LOADER_VERSION__"'
$content = $content -replace '"minecraft": "~[\d.]+"', '"minecraft": "~__MC_VERSION__"'
$content = $content -replace 'versionRange = "\[\d+\.\d+,\)"', 'versionRange = "[__NEOFORGE_MAJOR__,)"'
$content = $content -replace 'versionRange = "\[\d+\.\d+\.\d+,\)"', 'versionRange = "[__MC_VERSION__,)"'
Set-Content $File $content -NoNewline
