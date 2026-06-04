<#
.SYNOPSIS
    Remove demodatagen from Windows (binary, PATH entry, and optionally data).

.EXAMPLE
    .\uninstall.ps1

.EXAMPLE
    .\uninstall.ps1 -Purge -Yes
#>
[CmdletBinding()]
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\demodatagen\bin",
    [switch]$Purge,
    [switch]$Yes,
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"

function Write-Info($msg) { if (-not $Quiet) { Write-Host "==> $msg" -ForegroundColor Cyan } }
function Write-Note($msg) { if (-not $Quiet) { Write-Host "    $msg" } }
function Write-Warn($msg) { Write-Warning $msg }

$BinName = "demodatagen.exe"

# ── Locate installations ─────────────────────────────────────────────
$candidates = @(
    (Join-Path $InstallDir $BinName),
    (Join-Path "$env:USERPROFILE\.demodatagen\bin" $BinName)
)
$cmd = Get-Command demodatagen -ErrorAction SilentlyContinue
if ($cmd) { $candidates += $cmd.Source }

$found = @($candidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -Unique)

if ($found.Count -eq 0) {
    Write-Warn "no demodatagen installation found"
} else {
    Write-Info "Found $($found.Count) installation(s):"
    $found | ForEach-Object { Write-Note $_ }
    if (-not $Yes) {
        $reply = Read-Host "Remove these? [y/N]"
        if ($reply -notmatch '^[Yy]$') { Write-Error "aborted"; exit 1 }
    }
    foreach ($path in $found) {
        try { Remove-Item $path -Force; Write-Note "removed $path" }
        catch { Write-Warn "could not remove ${path}: $_" }
    }
}

# ── Remove from user PATH (and current session) ──────────────────────
$dirsToDrop = @($InstallDir, "$env:USERPROFILE\.demodatagen\bin")
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath) {
    $parts = @($userPath -split ';' | Where-Object { $_ -ne "" -and $dirsToDrop -notcontains $_ })
    $newPath = $parts -join ';'
    if ($newPath -ne $userPath) {
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path = ($env:Path -split ';' | Where-Object { $dirsToDrop -notcontains $_ }) -join ';'
        Write-Info "Removed demodatagen directories from user PATH."
    }
}

# ── Remove empty install directory ───────────────────────────────────
if ((Test-Path $InstallDir) -and -not (Get-ChildItem $InstallDir -Force)) {
    Remove-Item $InstallDir -Force
    Write-Note "removed empty $InstallDir"
}

# ── Purge config / data ──────────────────────────────────────────────
if ($Purge) {
    foreach ($dir in @(
        "$env:APPDATA\demodatagen",
        "$env:LOCALAPPDATA\demodatagen",
        "$env:USERPROFILE\.demodatagen")) {
        if (Test-Path $dir) {
            Remove-Item $dir -Recurse -Force -ErrorAction SilentlyContinue
            Write-Note "purged $dir"
        }
    }
}

Write-Info "demodatagen has been uninstalled."
