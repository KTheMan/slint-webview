param()

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-default-features
cargo test --workspace
cargo doc --workspace --all-features --no-deps
cargo package -p slint-webview-core --allow-dirty
cargo package -p slint-webview-core --allow-dirty --list
Write-Host "Facade crate package dry-run is blocked until slint-webview-core is published or vendored as part of a release."
