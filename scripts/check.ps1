param(
    [switch]$Smoke,
    [switch]$Visual
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --no-default-features
cargo test
cargo doc --all-features --no-deps
cargo check --features testing --bin slint-webview-regression

if ($Smoke) {
    cargo run --features testing --bin slint-webview-regression -- --smoke
}

if ($Visual) {
    & "$PSScriptRoot\capture-windows-visual.ps1"
}
