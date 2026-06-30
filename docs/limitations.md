# Limitations

## Native Child View

The current backend embeds a native child view. It is positioned inside the
native window but is not rendered by Slint.

Consequences:

- Slint clipping does not clip the webview.
- Slint transforms do not transform the webview.
- Slint opacity does not fade the webview.
- Slint overlays may appear behind the webview.
- Z-order behavior is platform-specific.

Applications should hide or move the webview when a Slint modal, menu, tooltip,
or overlay must appear above it. `WebViewAreaController` centralizes that policy
for the common case by parking hidden or overlay-covered webviews offscreen.

## Platform Differences

The same API maps to different native engines. Differences in focus handling,
download behavior, devtools, media support, and shutdown behavior should be
expected and tested per platform.

## Focus Semantics

The webview and Slint controls can both participate in native keyboard focus.
The crate exposes `focus`, `focus_parent`, `set_keyboard_focus_enabled`, and
focus events so applications can decide who owns typing. Explicit clicks between
webview inputs and Slint inputs are the supported focus transition path.
Hover-only focus retention can vary by backend, especially on X11/WSLg.

`WebViewAreaController` adds a higher-level contract: Slint inputs report
`shell_focus_active`, the controller disables webview keyboard focus while that
is true, and webview focus is re-enabled after a native focus request only when
the area is effectively visible.

## Runtime Requirements

Windows requires a WebView2 runtime. Linux requires GTK and WebKitGTK
development/runtime packages. macOS support depends on the platform WebKit
stack exposed through Wry.

## Not Yet Implemented

- True Slint scene-graph widget.
- Slint-owned web texture rendering.
- Servo backend.
- CEF backend.
- Public custom protocol API.
- Permission prompt interception.
- Runtime diagnostics for missing platform runtimes before attach.
- Verified macOS smoke coverage in this workspace.
