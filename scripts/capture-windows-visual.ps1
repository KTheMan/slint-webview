param(
    [int]$HoldSeconds = 8,
    [string]$OutputPath = "target\visual\smoke-windows.png"
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

cargo build --features testing --bin slint-webview-regression

$exe = Join-Path (Get-Location) "target\debug\slint-webview-regression.exe"
$resolvedOutput = Join-Path (Get-Location) $OutputPath
$outputDir = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$env:LIB = (($env:LIB -split ';') | Where-Object { $_ -and (Test-Path $_) }) -join ';'
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public static class SlintWebViewWin32
{
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
}
"@

function Save-WindowScreenshot {
    param(
        [IntPtr]$Handle,
        [string]$Path
    )

    $rect = New-Object SlintWebViewWin32+RECT
    if (-not [SlintWebViewWin32]::GetWindowRect($Handle, [ref]$rect)) {
        throw "GetWindowRect failed"
    }

    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -le 0 -or $height -le 0) {
        throw "invalid window bounds ${width}x${height}"
    }

    $bitmap = [System.Drawing.Bitmap]::new($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, [System.Drawing.Size]::new($width, $height))
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function Measure-VisualSentinels {
    param([string]$Path)

    $counts = @{
        Red = 0
        Teal = 0
        DarkWebview = 0
        WhiteSurface = 0
        ShellNavy = 0
    }

    $check = [System.Drawing.Bitmap]::new($Path)
    try {
        for ($x = 0; $x -lt $check.Width; $x += 4) {
            for ($y = 0; $y -lt $check.Height; $y += 4) {
                $pixel = $check.GetPixel($x, $y)
                if ($pixel.R -gt 220 -and $pixel.G -lt 90 -and $pixel.B -lt 90) {
                    $counts.Red++
                }
                if ($pixel.R -lt 90 -and $pixel.G -gt 150 -and $pixel.B -gt 140) {
                    $counts.Teal++
                }
                if ($pixel.R -ge 10 -and $pixel.R -le 35 -and $pixel.G -ge 25 -and $pixel.G -le 65 -and $pixel.B -ge 35 -and $pixel.B -le 85) {
                    $counts.DarkWebview++
                }
                if ($pixel.R -gt 235 -and $pixel.G -gt 235 -and $pixel.B -gt 235) {
                    $counts.WhiteSurface++
                }
                if ($pixel.R -ge 5 -and $pixel.R -le 25 -and $pixel.G -ge 30 -and $pixel.G -le 55 -and $pixel.B -ge 45 -and $pixel.B -le 75) {
                    $counts.ShellNavy++
                }
            }
        }
    } finally {
        $check.Dispose()
    }

    return $counts
}

function Test-VisualSentinels {
    param([hashtable]$Counts)

    return $Counts.Red -ge 5 -and $Counts.DarkWebview -ge 500 -and $Counts.WhiteSurface -ge 500 -and $Counts.ShellNavy -ge 500
}

$process = [System.Diagnostics.Process]::new()
$process.StartInfo.FileName = $exe
$process.StartInfo.Arguments = "--smoke --hold-seconds=$HoldSeconds"
$process.StartInfo.UseShellExecute = $false
$process.StartInfo.RedirectStandardOutput = $true
$process.StartInfo.RedirectStandardError = $true

if (-not $process.Start()) {
    throw "failed to start $exe"
}

try {
    $deadline = [DateTime]::UtcNow.AddSeconds([Math]::Max(8, $HoldSeconds))
    do {
        Start-Sleep -Milliseconds 250
        $process.Refresh()
        $handle = $process.MainWindowHandle
    } while ($handle -eq 0 -and [DateTime]::UtcNow -lt $deadline)

    if ($handle -eq 0) {
        throw "regression app window was not found"
    }

    Start-Sleep -Seconds ([Math]::Min(2, [Math]::Max(1, $HoldSeconds - 4)))

    $captureDeadline = [DateTime]::UtcNow.AddSeconds([Math]::Max(12, $HoldSeconds + 4))
    $counts = $null
    do {
        $process.Refresh()
        if ($process.HasExited) {
            break
        }

        Save-WindowScreenshot -Handle $process.MainWindowHandle -Path $resolvedOutput
        $counts = Measure-VisualSentinels -Path $resolvedOutput
        if (Test-VisualSentinels -Counts $counts) {
            break
        }

        Start-Sleep -Milliseconds 500
    } while ([DateTime]::UtcNow -lt $captureDeadline)

    if ($null -eq $counts) {
        throw "visual sentinel check failed: screenshot was not captured"
    }

    $red = $counts.Red
    $teal = $counts.Teal
    $darkWebview = $counts.DarkWebview
    $whiteSurface = $counts.WhiteSurface
    $shellNavy = $counts.ShellNavy

    if ($red -lt 5 -or $darkWebview -lt 500 -or $whiteSurface -lt 500 -or $shellNavy -lt 500) {
        throw "visual sentinel check failed: red=$red teal=$teal darkWebview=$darkWebview whiteSurface=$whiteSurface shellNavy=$shellNavy"
    }

    if (-not $process.WaitForExit(($HoldSeconds + 15) * 1000)) {
        throw "regression app did not exit"
    }

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    if ($stdout -notmatch "smoke-ok") {
        throw "smoke output did not include smoke-ok. stdout=$stdout stderr=$stderr"
    }
    if ($process.ExitCode -ne 0) {
        throw "regression app exited with $($process.ExitCode). stdout=$stdout stderr=$stderr"
    }

    Write-Host "visual-smoke-ok: $resolvedOutput"
    Write-Host "visual-sentinels: red=$red teal=$teal darkWebview=$darkWebview whiteSurface=$whiteSurface shellNavy=$shellNavy"
    Write-Host $stdout
} finally {
    if (-not $process.HasExited) {
        try {
            $process.Kill($true)
        } catch [System.Management.Automation.MethodException] {
            $process.Kill()
        }
    }
    $process.Dispose()
}
