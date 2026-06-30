# Changelog

All notable changes to this crate are documented here.

This project follows semantic versioning after the first public release. While
the crate remains `0.x`, API changes may still occur between minor versions.

## 0.1.0 - Release Candidate

Initial release candidate for native webview composition in Slint apps.

### Added

- `WebViewController` public API for attaching, moving, showing, hiding,
  focusing, loading, and scripting a native child webview.
- `WebViewAreaController` plus `ui/webview-area.slint` for Tier 1.5
  widget-style composition on top of the native child backend.
- `WebViewAreaPolicy`, `WebViewAreaState`, `WebViewAreaPlacement`, and
  `HiddenWebViewStrategy` for reusable parking, overlay, and focus policy.
- Wry backend using WebView2 on Windows, WebKitGTK on Linux, and WKWebView on
  macOS through Wry's platform support.
- Conservative `WebViewOptions` defaults with explicit opt-ins for JavaScript,
  devtools, clipboard, downloads, popups, incognito mode, initial focus, user
  agent override, initialization scripts, IPC limit, and navigation policy.
- Structured event stream for navigation, IPC, title changes, script results,
  focus signals, new-window requests, and download activity.
- Deterministic regression fixture and Slint regression app behind the
  `testing` feature.
- Windows and Linux/WSL verification scripts plus package dry-run scripts.
- Compileable `examples/area.rs` demonstrating `WebViewAreaController`.
- Documentation for architecture, API shape, security model, limitations,
  testing, licensing, and platform notes.
- `slint-webview-core` shared crate, concrete `native` backend crate, and
  compileable `servo` and `cef` backend crate shells.
- `WebViewBackend` and `BackendWebViewController` shared backend contract for
  native, Servo, and CEF implementations.
- `WebViewControllerLike` and `BackendWebViewAreaController` shared area
  contract so native, Servo, and CEF controllers can reuse the same Slint
  placeholder, overlay, parking, and focus policy.
- Wry implementation moved into `slint-webview-native`, with the root
  `slint-webview` crate acting as a facade over the native backend.

### Known Limitations

- The current composition tier is a native child view, not a true Slint
  scene-graph item.
- Slint clipping, transforms, opacity, and overlays do not automatically apply
  to the webview.
- WSLg/XWayland focus and shutdown behavior can differ from native Linux.
- macOS is API-shaped through Wry but not verified in this workspace.
