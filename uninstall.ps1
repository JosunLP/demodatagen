# uninstall.ps1 – Remove demodatagen from Windows.
#
# Usage (PowerShell):
#   .\uninstall.ps1

param(
    [string]$InstallDir = "$env:USERPROFILE\.demodatagen\bin"
)

$ErrorActionPreference = "Stop"

$Binary = Join-Path $InstallDir "demodatagen.exe"

if (Test-Path $Binary) {
    Remove-Item $Binary -Force
    Write-Host "Removed $Binary"
} else {
    Write-Host "demodatagen not found at $Binary"
    exit 1
}

# Remove from PATH if present
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -like "*$InstallDir*") {
    $NewPath = ($UserPath -split ";" | Where-Object { $_ -ne $InstallDir }) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "Removed $InstallDir from user PATH."
}

# Remove install directory if empty
if ((Test-Path $InstallDir) -and ((Get-ChildItem $InstallDir).Count -eq 0)) {
    Remove-Item $InstallDir -Force
    Write-Host "Removed empty directory $InstallDir"
}

Write-Host "demodatagen has been uninstalled."
