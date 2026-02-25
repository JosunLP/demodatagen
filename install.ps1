# install.ps1 – Install demodatagen on Windows from GitHub releases.
#
# Usage (PowerShell):
#   iwr -useb https://raw.githubusercontent.com/user/demodatagen/main/install.ps1 | iex
#   .\install.ps1 -Version v0.1.0

param(
    [string]$Version = "",
    [string]$InstallDir = "$env:USERPROFILE\.demodatagen\bin"
)

$ErrorActionPreference = "Stop"

$Repo = "user/demodatagen"
$Target = "x86_64-pc-windows-msvc"

# ── Resolve version ──────────────────────────────────────────────────
if (-not $Version) {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $release.tag_name
    if (-not $Version) {
        Write-Error "Could not determine latest version."
        exit 1
    }
}

Write-Host "Installing demodatagen $Version for $Target..."

# ── Download ─────────────────────────────────────────────────────────
$Url = "https://github.com/$Repo/releases/download/$Version/demodatagen-$Version-$Target.zip"
$TmpZip = Join-Path $env:TEMP "demodatagen.zip"
$TmpDir = Join-Path $env:TEMP "demodatagen_install"

Invoke-WebRequest -Uri $Url -OutFile $TmpZip

if (Test-Path $TmpDir) { Remove-Item $TmpDir -Recurse -Force }
Expand-Archive -Path $TmpZip -DestinationPath $TmpDir

# ── Install ──────────────────────────────────────────────────────────
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Copy-Item "$TmpDir\demodatagen.exe" "$InstallDir\demodatagen.exe" -Force

# ── Add to PATH ──────────────────────────────────────────────────────
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to user PATH."
}

# ── Cleanup ──────────────────────────────────────────────────────────
Remove-Item $TmpZip -Force -ErrorAction SilentlyContinue
Remove-Item $TmpDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "demodatagen $Version installed to $InstallDir\demodatagen.exe"
Write-Host "Run 'demodatagen --help' to get started."
