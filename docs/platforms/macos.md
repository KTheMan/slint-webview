# macOS

macOS support is expected to use WKWebView through Wry's platform backend.

## Current Status

The public API is shaped for macOS, but this workspace has not run a macOS
smoke test. Do not claim macOS production coverage until the regression app is
built and exercised on macOS.

## Expected Requirements

- Rust toolchain for macOS.
- A Slint native window handle available before `WebViewController::attach`.
- The platform WebKit stack available through WKWebView.

## Verification To Add

Run the equivalent of:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run --features testing --bin slint-webview-regression -- --smoke
```

Manual checks should include focus transfer between Slint inputs and webview
inputs, resizing, show/hide, modal hide policy, navigation policy, IPC, and
script evaluation.
