param(
    [switch]$Build
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

if ($Build) {
    cargo build --features testing --bin slint-webview-regression
}

$exe = Join-Path (Get-Location) "target\debug\slint-webview-regression.exe"
if (-not (Test-Path $exe)) {
    cargo build --features testing --bin slint-webview-regression
}

$process = Start-Process -FilePath $exe -PassThru
Write-Host "windows-pid=$($process.Id)"
