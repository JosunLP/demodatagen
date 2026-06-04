<#
.SYNOPSIS
    Install the demodatagen binary on Windows from GitHub releases.

.DESCRIPTION
    Downloads the latest (or a specified) release, verifies its SHA-256
    checksum against the published SHA256SUMS, installs it, and adds the
    install directory to the user PATH (and the current session).

.EXAMPLE
    iwr -useb https://raw.githubusercontent.com/j-pfalzgraf/demodatagen/main/install.ps1 | iex

.EXAMPLE
    .\install.ps1 -Version v0.2.0 -Force
#>
[CmdletBinding()]
param(
    [string]$Version = $env:DEMODATAGEN_VERSION,
    [string]$Repo = $(if ($env:DEMODATAGEN_REPO) { $env:DEMODATAGEN_REPO } else { "j-pfalzgraf/demodatagen" }),
    [string]$InstallDir = "$env:LOCALAPPDATA\demodatagen\bin",
    [switch]$NoModifyPath,
    [switch]$Force,
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"
# Force TLS 1.2 for older Windows PowerShell.
try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 } catch {}

function Write-Info($msg) { if (-not $Quiet) { Write-Host "==> $msg" -ForegroundColor Cyan } }
function Write-Note($msg) { if (-not $Quiet) { Write-Host "    $msg" -ForegroundColor DarkGray } }
function Write-Warn($msg) { Write-Warning $msg }
function Die($msg) { Write-Error $msg; exit 1 }

# ── Detect architecture ──────────────────────────────────────────────
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq "ARM64") {
    Write-Note "Detected ARM64; installing the x86_64 build (runs via emulation)."
}
# Only an x86_64 Windows asset is published; it runs on ARM64 via emulation.
$Target = "x86_64-pc-windows-msvc"
$BinName = "demodatagen.exe"

# ── Resolve version ──────────────────────────────────────────────────
if (-not $Version) {
    Write-Info "Querying latest release of $Repo..."
    try {
        $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest" `
            -Headers @{ "User-Agent" = "demodatagen-install" }
        $Version = $release.tag_name
    } catch {
        Die "Could not determine latest version: $_"
    }
    if (-not $Version) { Die "Could not determine latest version (no releases?)." }
}
if ($Version -notlike "v*") { $Version = "v$Version" }

Write-Info "Installing demodatagen $Version ($Target)"

# ── Download ─────────────────────────────────────────────────────────
$Archive = "demodatagen-$Version-$Target.zip"
$BaseUrl = "https://github.com/$Repo/releases/download/$Version"
$TmpDir  = Join-Path $env:TEMP ("demodatagen_install_" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null
try {
    $ZipPath = Join-Path $TmpDir $Archive
    Write-Note "downloading $Archive"
    Invoke-WebRequest -Uri "$BaseUrl/$Archive" -OutFile $ZipPath -UseBasicParsing `
        -Headers @{ "User-Agent" = "demodatagen-install" }

    # ── Verify checksum ──────────────────────────────────────────────
    $SumsPath = Join-Path $TmpDir "SHA256SUMS"
    try {
        Invoke-WebRequest -Uri "$BaseUrl/SHA256SUMS" -OutFile $SumsPath -UseBasicParsing `
            -Headers @{ "User-Agent" = "demodatagen-install" }
    } catch { $SumsPath = $null }

    if ($SumsPath -and (Test-Path $SumsPath)) {
        $expected = $null
        foreach ($line in Get-Content $SumsPath) {
            if ($line -match "\b([0-9a-fA-F]{64})\b" -and $line -match [regex]::Escape($Archive)) {
                $expected = $Matches[1].ToLower(); break
            }
        }
        $actual = (Get-FileHash -Algorithm SHA256 $ZipPath).Hash.ToLower()
        if (-not $expected) {
            Write-Warn "checksum for $Archive not found in SHA256SUMS; skipping verification"
        } elseif ($expected -ne $actual) {
            Die "checksum mismatch for ${Archive}: expected $expected, got $actual"
        } else {
            Write-Note "checksum OK"
        }
    } else {
        Write-Warn "SHA256SUMS not published for $Version; skipping checksum verification"
    }

    # ── Extract ──────────────────────────────────────────────────────
    $ExtractDir = Join-Path $TmpDir "extract"
    Expand-Archive -Path $ZipPath -DestinationPath $ExtractDir -Force
    $src = Get-ChildItem -Path $ExtractDir -Recurse -Filter $BinName | Select-Object -First 1
    if (-not $src) { Die "binary $BinName not found inside $Archive" }

    # ── Install ──────────────────────────────────────────────────────
    $dest = Join-Path $InstallDir $BinName
    if ((Test-Path $dest) -and (-not $Force)) {
        $reply = Read-Host "$dest already exists. Overwrite? [y/N]"
        if ($reply -notmatch '^[Yy]$') { Die "aborted" }
    }
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    Copy-Item $src.FullName $dest -Force

    # ── Verify it runs ───────────────────────────────────────────────
    try {
        $ver = & $dest --version 2>$null
        Write-Info "Installed $ver -> $dest"
    } catch {
        Write-Warn "installed binary did not run cleanly: $_"
    }

    # ── PATH setup ───────────────────────────────────────────────────
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @($userPath -split ';' | Where-Object { $_ -ne "" })
    if ($parts -notcontains $InstallDir) {
        if (-not $NoModifyPath) {
            $newPath = (@($parts) + $InstallDir) -join ';'
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            $env:Path = "$env:Path;$InstallDir"   # current session too
            Write-Info "Added $InstallDir to your user PATH."
        } else {
            Write-Warn "$InstallDir is not on PATH. Add it manually or omit -NoModifyPath."
        }
    }

    Write-Info "Done. Try: demodatagen list"
}
finally {
    Remove-Item $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
