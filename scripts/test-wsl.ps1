param(
    [switch]$Smoke
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

$windowsPath = (Get-Location).Path
$linuxPath = (wsl.exe wslpath -a "$windowsPath") -join ""
$escapedPath = $linuxPath.Replace("'", "'\''")

$scriptArgs = ""
if ($Smoke) {
    $scriptArgs = " --smoke"
}

wsl.exe bash -lc "cd '$escapedPath' && bash ./scripts/test-wsl.sh$scriptArgs"
exit $LASTEXITCODE
