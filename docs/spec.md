# Technical Specification

## Public API

The crate exposes a controller-oriented API:

- `WebViewController`: owns the native backend instance and event receiver.
- `WebViewAreaController`: wraps `WebViewController` with Slint area
  synchronization, parking, overlay, and focus policy.
- `WebViewAreaPolicy`, `WebViewAreaState`, and `WebViewAreaPlacement`:
  serializable policy/state types for widget-style composition.
- `WebViewBackend` and `BackendWebViewController`: shared backend trait and
  backend-agnostic controller used by concrete backend crates.
- `WebViewControllerLike` and `BackendWebViewAreaController`: shared
  controller-facing trait and area controller used to keep Slint placeholder
  policy consistent across backend families.
- `RenderedWebViewBackend`, `BackendRenderedWebViewController`,
  `RenderedWebViewFrame`, and `RenderedWebViewInputEvent`: shared rendered
  backend contract for Servo and CEF style Slint-owned composition.
- `WebViewOptions`: creation-time configuration.
- `WebViewSource`: initial content selection.
- `WebViewBounds`: Slint logical-pixel rectangle.
- `WebViewEvent`: structured event stream from web content and backend hooks.
- `WebViewCapabilities`: static backend capability description.

The backend module remains private. Application code should not depend on Wry
types directly.

The crate also ships `ui/webview-area.slint`, a declarative placeholder
component for apps that want the webview represented in their Slint UI tree.

Backend crates should implement the same public contract from
`slint-webview-core`. The workspace now contains `slint-webview-native`,
`slint-webview-servo`, and `slint-webview-cef`, with `slint-webview` remaining
the facade crate. The native crate owns the Wry backend; Servo and CEF remain
backend shells.

## Lifecycle

1. Call `initialize_platform()` once before attaching webviews. On Linux this
   initializes GTK.
2. Create and show the Slint window so a native window handle exists.
3. Create `WebViewOptions` with explicit source, bounds, and enabled
   capabilities.
4. Call `WebViewController::attach(&window_handle, options)`.
5. Keep the controller alive while the native child view should remain attached.
6. Call `pump_platform_events()` from the Slint tick/timer path where required.
7. Drain controller events regularly with `try_recv_event()` or `drain_events()`.
8. Drop the controller before or during window teardown.

For `WebViewAreaController`, replace steps 4, 6, and 7 with:

1. Create `WebViewAreaState` from the Slint placeholder bounds and UI state.
2. Call `WebViewAreaController::attach(&window_handle, options, state, policy)`.
3. Call `sync(state)` on state changes, or `tick(state)` from a Slint timer to
   pump platform events, synchronize placement, drain events, and apply focus
   event policy.

## Coordinate Model

`WebViewBounds` uses Slint logical window coordinates. The backend translates
them to Wry's rectangle type and lets Wry/platform code handle the native scale
conversion. Bounds must be finite and have positive width and height.

## Security Defaults

Default options load a blank document and disable:

- JavaScript
- Devtools
- Clipboard access
- Popups
- Downloads
- Initial webview focus
- Incognito/private profile mode

Callers must explicitly enable each capability they need.

## Event Model

Events are delivered over a per-controller channel:

- Navigation requests and load lifecycle
- IPC messages with URI and truncation status
- Document title changes
- Script result events correlated by `ScriptRequestId`
- Popup requests with allow/deny result
- Download requests and completion reports

The event channel is intentionally pull-based so Slint apps can drain it from
their normal UI tick/timer path.

## Backend Contract

The first backend is Wry. It uses native child surfaces:

- Windows: WebView2
- Linux: WebKitGTK
- macOS: WKWebView through Wry

The native child surface is not part of the Slint scene graph. The application
must manage overlay policy by hiding or moving the webview when Slint content
needs to appear above it.

`WebViewAreaPolicy::default()` uses offscreen parking for hidden webviews. Apps
may opt into native hide or hide-and-park if that fits their platform behavior
better.

Backend crates implement `WebViewBackend` for their concrete engine surface.
`BackendWebViewController` then supplies uniform event draining, script request
ID allocation, source loading dispatch, and bounds validation.

Backend crates should expose a controller implementing `WebViewControllerLike`
and compose it with `BackendWebViewAreaController` for uniform hiding/parking,
overlay, shell-focus, and focus-request behavior.

Servo and CEF backends should report `CompositionTier::SlintOwnedTexture` once
they render into buffers or textures owned by Slint. They should implement
`RenderedWebViewBackend` for surface resizing, Slint-originated input events,
and frame production, then wrap the engine instance in
`BackendRenderedWebViewController` for shared browser dispatch and rendered
frame validation. A Windows-only WebView2 Visual Hosting backend should report
`CompositionTier::PlatformVisualHosting`.

## Feature Flags

- `backend-wry`: enables the native Wry backend. This is the default feature.
- `testing`: exposes fixture helpers and the regression app.

`cargo test --workspace --no-default-features` verifies that the public API, the
disabled native stub, and backend crate shells still compile when no native
backend is selected.
