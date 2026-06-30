param(
    [switch]$Smoke,
    [switch]$Visual
)

$ErrorActionPreference = "Stop"

$arguments = @()
if ($Smoke) {
    $arguments += "-Smoke"
}
if ($Visual) {
    $arguments += "-Visual"
}

& "$PSScriptRoot\check.ps1" @arguments
