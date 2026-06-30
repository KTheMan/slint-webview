#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo fmt --check
cargo test --workspace --no-default-features

if command -v pkg-config >/dev/null 2>&1 \
  && pkg-config --exists gtk+-3.0 \
  && pkg-config --exists webkit2gtk-4.1; then
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test --workspace
  cargo doc --workspace --all-features --no-deps
  cargo package -p slint-webview-core --allow-dirty
  cargo package -p slint-webview-core --allow-dirty --list
  echo "Facade crate package dry-run is blocked until slint-webview-core is published or vendored as part of a release."
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
