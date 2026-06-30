param(
    [switch]$Smoke,
    [switch]$Visual
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-default-features
cargo test --workspace
cargo doc --workspace --all-features --no-deps
cargo check --features testing --bin slint-webview-regression

if ($Smoke) {
    cargo run --features testing --bin slint-webview-regression -- --smoke
}

if ($Visual) {
    & "$PSScriptRoot\capture-windows-visual.ps1"
}
