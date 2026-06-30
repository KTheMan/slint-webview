# Windows

Windows uses WebView2 through the Wry backend.

## Requirements

- Rust toolchain for Windows.
- WebView2 Runtime installed on the machine.
- Slint native window handle available before `WebViewController::attach`.

## Verification

```powershell
.\scripts\check.ps1
.\scripts\check.ps1 -Smoke
.\scripts\check.ps1 -Smoke -Visual
```

The visual smoke writes `target/visual/smoke-windows.png` and checks sentinel
colors so the result is not merely a blank native window.

## WebView2 Debugging

For CDP-based automation or inspection:

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
cargo run --features testing --bin slint-webview-regression -- --smoke
```

Use this only in trusted test environments.
