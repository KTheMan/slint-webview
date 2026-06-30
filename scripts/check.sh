#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

smoke=0
for arg in "$@"; do
  case "$arg" in
    --smoke)
      smoke=1
      ;;
    *)
      echo "unknown argument: $arg" >&2
      exit 64
      ;;
  esac
done

cargo fmt --check
cargo test --no-default-features

if command -v pkg-config >/dev/null 2>&1 \
  && pkg-config --exists gtk+-3.0 \
  && pkg-config --exists webkit2gtk-4.1; then
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test
  cargo doc --all-features --no-deps
  cargo check --features testing --bin slint-webview-regression

  if [[ "$smoke" -eq 1 ]]; then
    cargo build --features testing --bin slint-webview-regression
    SLINT_BACKEND="${SLINT_BACKEND:-winit-software}" \
      GDK_BACKEND="${GDK_BACKEND:-x11}" \
      WINIT_UNIX_BACKEND="${WINIT_UNIX_BACKEND:-x11}" \
      DISPLAY="${DISPLAY:-:0}" \
      XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/mnt/wslg/runtime-dir}" \
      PULSE_SERVER="${PULSE_SERVER:-/mnt/wslg/PulseServer}" \
      timeout 75s target/debug/slint-webview-regression --smoke
  fi
else
  cat <<'MSG'
Linux native webview dependencies are missing.
Install them on Ubuntu with:
  sudo apt update
  sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
Pure Rust API tests passed with --no-default-features.
MSG

  if [[ "$smoke" -eq 1 ]]; then
    exit 20
  fi
fi
