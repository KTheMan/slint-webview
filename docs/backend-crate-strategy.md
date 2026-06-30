# Backend Crate Strategy

This workspace uses a shared core plus backend-specific crate shells. The goal
is to keep the Slint API, events, security defaults, tests, and component shape
uniform while allowing each rendering engine to solve only its final
platform-specific integration step.

## Package Topology

Package split:

| Crate | Role | Composition target |
| --- | --- | --- |
| `slint-webview-core` | Shared API, policy, events, test fixtures, Slint component assets, backend traits | No backend |
| `slint-webview-native` | Wry-backed OS webviews: WebView2, WKWebView, WebKitGTK | `NativeChildView`, with optional platform visual hosting |
| `slint-webview-servo` | Servo-backed renderer integrated through Slint texture/compositor paths | `SlintOwnedTexture` |
| `slint-webview-cef` | CEF windowless/offscreen Chromium backend | `SlintOwnedTexture`, with platform-specific accelerated paths |
| `slint-webview` | Convenience facade crate selecting one backend with features | Depends on core plus one backend |

The workspace now contains `slint-webview-core` plus shell crates for native,
Servo, and CEF. The current Wry implementation still lives in the root
`slint-webview` facade crate until the backend contract is stable enough to move
that code into `slint-webview-native` without disrupting examples and tests.

## Backend Contract

All backend crates should implement the same host-facing behavior:

- Attach/create a webview from `WebViewOptions`.
- Load `WebViewSource::Blank`, `Url`, and `Html`.
- Apply logical bounds from Slint.
- Apply requested visibility and focusability.
- Evaluate JavaScript and correlate results by `ScriptRequestId`.
- Emit the common `WebViewEvent` stream.
- Enforce conservative defaults for JavaScript, clipboard, downloads, popups,
  devtools, and initial focus.
- Report `WebViewCapabilities` truthfully.

The backend boundary should avoid exposing Wry, Servo, CEF, WebView2, WebKitGTK,
or WKWebView types through the shared API.

## Composition Families

### Native

`slint-webview-native` should target small app size and platform fidelity. It
uses the webview engine already supplied by the platform:

- Windows: WebView2 through Wry.
- macOS: WKWebView through Wry.
- Linux: WebKitGTK through Wry.

This family is the best default when the app needs a small dependency footprint
and can tolerate native-child-view limitations. It is not expected to provide
identical z-order, clipping, focus, or monitor-move behavior across platforms.

Future native work can include a Windows WebView2 Visual Hosting backend. That
would still be native, but it should report `PlatformVisualHosting` rather than
`NativeChildView` because the host app would manage composition and spatial
input more directly.

### Servo

`slint-webview-servo` should target Rust-native, Slint-composited rendering. It
is the best long-term fit for a true Slint widget because Servo can render into
GPU surfaces that Slint can present.

This backend should prioritize:

- Shared texture or zero-copy paths where Slint and Servo can share GPU memory.
- A CPU pixel-buffer fallback only for debugging or platforms without a shared
  texture path.
- Common input translation from Slint pointer, wheel, keyboard, IME, clipboard,
  and focus events into Servo.
- Clear compatibility labeling because Servo's web-platform coverage is still
  different from Chromium and platform WebKit.

### CEF

`slint-webview-cef` should target the most consistent production web behavior.
CEF windowless/offscreen rendering can avoid native child windows and let Slint
own composition, but it brings Chromium's binary size, process model, packaging,
and update burden.

This backend should prioritize:

- CPU paint-buffer support first, because it is the most portable baseline.
- Accelerated paint/shared-texture paths per platform after the baseline works.
- Explicit packaging docs for CEF binaries and subprocesses.
- A security/update policy, because shipping Chromium means inheriting browser
  patch cadence responsibilities.

## Shared Test Harness

Backends should share the same deterministic fixture and smoke contracts:

- Attach and first-paint readiness.
- DOM probe and script evaluation.
- IPC round trip.
- Navigation allow/block policy.
- Popup allow/block policy.
- Download allow/block policy.
- Focus transfer between Slint controls and web content.
- Hide/show and modal overlay behavior.
- Resize and monitor-move behavior.
- Visual sentinel capture for Slint UI plus web content.

The fixture should live in `slint-webview-core` so every backend proves the same
observable behavior.

## Migration Plan

1. Keep the current crate unpublished as `slint-webview` until the owner
   decides release policy.
2. Keep shared API, event, fixture, capability, error, and area-policy types in
   `slint-webview-core`.
3. Introduce an internal backend trait behind the existing controller without
   changing public API.
4. Move Wry code into `slint-webview-native` and make the facade depend on it
   by default.
5. Add `slint-webview-servo` behind an opt-in feature or separate example app.
6. Add `slint-webview-cef` after Servo or in parallel if Chromium compatibility
   is the priority.
7. Promote backend-agnostic examples and tests so all three backend crates run
   the same contract.

## Decision Guidance

If the priority is small install size, ship native first.

If the priority is identical composition behavior, build Servo and CEF as
texture/offscreen backends.

If the priority is maximum web compatibility, CEF is the faster consistency win.

If the priority is Rust-native architecture and deep Slint integration, Servo is
the better long-term bet.
