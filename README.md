# slint-webview

Native webview controller for Slint applications.

`slint-webview` embeds the platform webview as a native child surface inside a
Slint window. The default backend is Wry, which maps to WebView2 on Windows,
WebKitGTK on Linux, and WKWebView on macOS through the platform support exposed
by Wry.

This crate is release-candidate shaped, but crates.io publishing is
intentionally blocked with `publish = false` until the owner decides to cut a
public package release. Older webview experiments elsewhere in the larger
workspace are non-authoritative unless a maintainer explicitly points to them.

## What This Is

- A Rust workspace with shared `slint-webview-core` types and a root
  `slint-webview` facade/native crate.
- A shared `WebViewBackend` trait and `BackendWebViewController` for backend
  implementations.
- A Rust library API centered on `WebViewController`.
- A Tier 1.5 `WebViewArea`/`WebViewAreaController` composition layer for Slint
  apps that want reusable parking, overlay, and focus policy.
- A Wry-backed native child-view implementation.
- Compileable crate shells for `slint-webview-native`, `slint-webview-servo`,
  and `slint-webview-cef`.
- Conservative defaults: blank page, JavaScript off, devtools off, clipboard
  off, downloads off, popups off, initial webview focus off.
- Structured events for navigation, IPC, title changes, script results, popups,
  and downloads.
- A deterministic Slint regression app behind the `testing` feature.
- Windows and WSL/Linux verification scripts.
- Release packaging scripts and a compileable minimal example.

## What This Is Not

- Not a true Slint scene-graph item yet.
- Not a texture-rendered web engine.
- Not an overlay-safe or clipped widget on every platform.
- Not a fully verified macOS release until a WKWebView smoke run is completed
  on macOS.

The current composition tier is a native child view. That keeps binaries smaller
than embedding a full engine such as Servo, but it means Slint clipping,
transforms, opacity, z-order, and modal overlays need explicit application
policy. See [docs/limitations.md](docs/limitations.md).

## Minimal Usage

Create the Slint window first, show it so the native handle exists, then attach
the webview:

```rust,no_run
use slint_webview::{
    WebViewBounds, WebViewController, WebViewOptions, WebViewSource,
    initialize_platform, pump_platform_events,
};

fn attach_example<W>(window_handle: &W) -> slint_webview::Result<WebViewController>
where
    W: raw_window_handle::HasWindowHandle,
{
    initialize_platform()?;

    let options = WebViewOptions::default()
        .with_source(WebViewSource::Html("<h1>Hello from the webview</h1>".to_owned()))
        .with_bounds(WebViewBounds::new(300.0, 64.0, 640.0, 480.0))
        .with_javascript_enabled(true);

    WebViewController::attach(window_handle, options)
}

fn tick(controller: &WebViewController) {
    pump_platform_events();

    for event in controller.drain_events() {
        eprintln!("{event:?}");
    }
}
```

`WebViewBounds` uses Slint logical window coordinates. Keep the controller alive
for as long as the native child webview should exist.

See [examples/minimal.rs](examples/minimal.rs) for a compileable starter app.

## WebViewArea Composition

For app code that wants a Slint-facing widget surface, import
`ui/webview-area.slint` and drive it with `WebViewAreaController`.

```rust,no_run
use slint_webview::{
    WebViewAreaController, WebViewAreaPolicy, WebViewAreaState, WebViewBounds,
    WebViewOptions,
};

fn attach_area<W>(window_handle: &W) -> slint_webview::Result<WebViewAreaController>
where
    W: raw_window_handle::HasWindowHandle,
{
    let state = WebViewAreaState::new(WebViewBounds::new(276.0, 72.0, 600.0, 504.0));
    WebViewAreaController::attach(
        window_handle,
        WebViewOptions::default().with_bounds(state.bounds),
        state,
        WebViewAreaPolicy::default(),
    )
}
```

The default area policy parks hidden or overlay-covered webviews offscreen. This
keeps Slint modals and shell inputs usable on platforms where native hide/focus
behavior is slow or inconsistent.

See [examples/area.rs](examples/area.rs) and
[docs/webview-area.md](docs/webview-area.md).

## Backend Direction

The repository now has a shared core plus backend crate shells:

- `slint-webview-core` for API types, events, policy, fixtures, and Slint
  component assets.
- `slint-webview-native` for Wry-backed WebView2, WKWebView, and WebKitGTK.
- `slint-webview-servo` for Servo-backed Slint texture composition.
- `slint-webview-cef` for CEF windowless/offscreen Chromium composition.
- `slint-webview` as a convenience facade.

Native remains the smallest platform-integrated path. Servo and CEF are the
consistency paths because they can target Slint-owned texture/offscreen
composition. The Wry implementation still lives in the facade crate until it is
mechanically moved into `slint-webview-native`. See
[docs/backend-crate-strategy.md](docs/backend-crate-strategy.md).

## Installation

For Git-based use while this crate remains unpublished on crates.io:

```toml
[dependencies]
slint-webview = { git = "https://github.com/KTheMan/slint-webview" }
```

For local path development:

```toml
slint-webview = { path = "../slint-webview" }
```

To compile only the public API types without native webview dependencies:

```toml
slint-webview = { git = "https://github.com/KTheMan/slint-webview", default-features = false }
```

## Feature Flags

| Feature | Default | Purpose |
| --- | --- | --- |
| `backend-wry` | yes | Enables the native Wry backend |
| `testing` | no | Enables fixture helpers and the regression app |

`backend-wry` pulls in platform webview dependencies for the root facade crate.
On Linux, GTK/WebKitGTK development packages are required only when this feature
is enabled.

## Platform Matrix

| Platform | Native engine | Status |
| --- | --- | --- |
| Windows | WebView2 | Smoke and visual checks pass locally |
| Linux/WSL | WebKitGTK | Smoke check passes when GTK/WebKitGTK deps are installed |
| macOS | WKWebView via Wry | API-shaped, not verified in this workspace |

## Verification

Release package dry-run:

```powershell
.\scripts\package.ps1
```

or:

```bash
bash scripts/package.sh
```

Windows:

```powershell
.\scripts\check.ps1
.\scripts\check.ps1 -Smoke
.\scripts\check.ps1 -Smoke -Visual
.\scripts\watch-tests.ps1 -Smoke
.\scripts\launch-windows-regression.ps1 -Build
```

WSL/Linux:

```powershell
.\scripts\test-wsl.ps1
.\scripts\test-wsl.ps1 -Smoke
wsl.exe --cd "$(wsl.exe wslpath -a (Get-Location).Path)" bash -lc "bash scripts/launch-wsl-regression.sh --build"
```

For native Linux builds, install the webview dependencies if the script reports
they are missing:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

For Playwright/WebView2 CDP automation, set the WebView2 debug environment
before launching the regression app:

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
cargo run --features testing --bin slint-webview-regression -- --smoke
```

## Documentation

- [Product requirements](docs/prd.md)
- [Technical specification](docs/spec.md)
- [Architecture](docs/architecture.md)
- [API notes](docs/api.md)
- [Backend crate strategy](docs/backend-crate-strategy.md)
- [Security model](docs/security.md)
- [Testing strategy](docs/testing.md)
- [WebViewArea composition](docs/webview-area.md)
- [Limitations](docs/limitations.md)
- [Windows notes](docs/platforms/windows.md)
- [Linux/WSL notes](docs/platforms/linux-wsl.md)
- [macOS notes](docs/platforms/macos.md)
- [Production readiness audit](docs/production-readiness-audit.md)
- [Licensing](docs/licensing.md)
- [Release checklist](RELEASE.md)
- [Changelog](CHANGELOG.md)

## Release Status

`0.1.0` is a release candidate. Workspace gates pass locally, and the
`slint-webview-core` package dry-run passes. The facade package dry-run is
blocked until `slint-webview-core` is published or deliberately vendored for a
release. `publish = false` remains set.

## License

This crate is licensed under `LGPL-3.0-only`. See
[docs/licensing.md](docs/licensing.md), [COPYING.LESSER](COPYING.LESSER), and
[COPYING](COPYING).

Dependencies keep their own licenses. Slint's non-GPL license paths remain a
separate downstream responsibility.
