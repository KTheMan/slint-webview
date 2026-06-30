param(
    [int]$Seconds = 15,
    [switch]$Smoke,
    [switch]$Visual,
    [switch]$Wsl
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

while ($true) {
    $stamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$stamp] running slint-webview regression checks"
    try {
        if ($Visual) {
            & "$PSScriptRoot\test-windows.ps1" -Smoke:$Smoke -Visual
        } else {
            & "$PSScriptRoot\test-windows.ps1" -Smoke:$Smoke
        }

        if ($Wsl) {
            & "$PSScriptRoot\test-wsl.ps1" -Smoke:$Smoke
        }
        Write-Host "checks passed"
    } catch {
        Write-Host "checks failed: $($_.Exception.Message)" -ForegroundColor Red
    }
    Start-Sleep -Seconds $Seconds
}
