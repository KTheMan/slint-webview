# Linux And WSL

Linux uses WebKitGTK through the Wry backend. The local smoke path has been
verified under WSL with WSLg when the GTK/WebKitGTK dependencies are installed.

## Dependencies

On Ubuntu:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

Without these packages, `scripts/check.sh` still runs
`cargo test --no-default-features` and reports that native webview dependencies
are missing.

## Verification From Windows

```powershell
.\scripts\test-wsl.ps1
.\scripts\test-wsl.ps1 -Smoke
```

The smoke command sets WSL-friendly defaults for `SLINT_BACKEND`, `GDK_BACKEND`,
`WINIT_UNIX_BACKEND`, `DISPLAY`, `XDG_RUNTIME_DIR`, and `PULSE_SERVER` before
launching the regression app.

## Known WSL Quirk

WSL/XWayland may report a shutdown-time `GLXBadWindow` warning after the smoke
probe has already succeeded. The harness treats this as a post-success
environment warning, not as proof of a production Linux shutdown guarantee.

WSLg focus routing is also more sensitive than native Windows WebView2. The
regression app now restores focus correctly after explicit clicks between Slint
and webview text fields, but pointer-hover focus retention between the two
native surfaces remains best-effort. Treat WSL as an important regression
environment, not as a perfect proxy for native Linux focus behavior.

When a WSLg window is moved between monitors, the backend periodically re-syncs
the WebKitGTK child X11 window using the webview window ancestry. This reduces
the common off-monitor or stale-size drift, but multi-monitor WSLg composition
should still receive manual release validation.
