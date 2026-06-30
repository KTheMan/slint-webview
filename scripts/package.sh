#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo fmt --check
cargo test --no-default-features

if command -v pkg-config >/dev/null 2>&1 \
  && pkg-config --exists gtk+-3.0 \
  && pkg-config --exists webkit2gtk-4.1; then
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test
  cargo doc --all-features --no-deps
  cargo package --allow-dirty
  cargo package --allow-dirty --list
else
  cat <<'MSG'
Linux native webview dependencies are missing.
Install them on Ubuntu with:
  sudo apt update
  sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
Only no-default-features tests were run.
MSG
  exit 20
fi
