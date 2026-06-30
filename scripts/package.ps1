param()

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --no-default-features
cargo test
cargo doc --all-features --no-deps
cargo package --allow-dirty
cargo package --list
