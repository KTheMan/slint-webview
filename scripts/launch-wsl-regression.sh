#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ "${1:-}" == "--build" ]]; then
  cargo build --features testing --bin slint-webview-regression
fi

if [[ ! -x target/debug/slint-webview-regression ]]; then
  cargo build --features testing --bin slint-webview-regression
fi

export SLINT_BACKEND="${SLINT_BACKEND:-winit-software}"
export GDK_BACKEND="${GDK_BACKEND:-x11}"
export WINIT_UNIX_BACKEND="${WINIT_UNIX_BACKEND:-x11}"
export DISPLAY="${DISPLAY:-:0}"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/mnt/wslg/runtime-dir}"
export PULSE_SERVER="${PULSE_SERVER:-/mnt/wslg/PulseServer}"

exec target/debug/slint-webview-regression
