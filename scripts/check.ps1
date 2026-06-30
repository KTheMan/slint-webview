param(
    [switch]$Smoke,
    [switch]$Visual
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Command,
        [string[]]$Arguments
    )

    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Command $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
}

Invoke-Checked "cargo" @("fmt", "--check")
Invoke-Checked "cargo" @("clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings")
Invoke-Checked "cargo" @("test", "--workspace", "--no-default-features")
Invoke-Checked "cargo" @("test", "--workspace")
Invoke-Checked "cargo" @("doc", "--workspace", "--all-features", "--no-deps")
Invoke-Checked "cargo" @("check", "--features", "testing", "--bin", "slint-webview-regression")

if ($Smoke) {
    Invoke-Checked "cargo" @("run", "--features", "testing", "--bin", "slint-webview-regression", "--", "--smoke")
}

if ($Visual) {
    & "$PSScriptRoot\capture-windows-visual.ps1"
}
