param()

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
Invoke-Checked "cargo" @("package", "-p", "slint-webview-core", "--allow-dirty")
Invoke-Checked "cargo" @("package", "-p", "slint-webview-core", "--allow-dirty", "--list")
Invoke-Checked "cargo" @("package", "-p", "slint-webview-mock", "--allow-dirty", "--list")
Write-Host "Facade and mock crate package dry-runs are blocked until path dependencies such as slint-webview-core and slint-webview-native are published or vendored as part of a release."
