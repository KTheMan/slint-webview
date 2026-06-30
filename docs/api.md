# API Notes

## `WebViewController`

`WebViewController` is the primary type. It owns the native backend and the
event receiver. Dropping it drops the native webview.

Important methods:

- `attach`: creates the native child webview.
- `set_bounds`: moves/resizes the webview in logical pixels.
- `set_visible`: shows or hides the native child view.
- `load_source`, `load_html`, `load_url`: navigate or replace content.
- `evaluate_script`: returns a `ScriptRequestId` for matching result events.
- `focus`, `focus_parent`: move keyboard focus between webview and host.
- `set_keyboard_focus_enabled`: gates whether the webview may take keyboard
  focus. Linux/WebKitGTK uses this to prevent hover-driven focus theft while a
  Slint text input owns typing; unsupported platforms treat it as a no-op.
- `drain_events`: pulls all currently queued events.

## `WebViewOptions`

Options are plain data and also provide builder-style helpers. Defaults are
conservative and do not load fixture content.

Use explicit opt-ins for:

- JavaScript
- Devtools
- Initial webview focus
- Clipboard
- Downloads
- Popups
- Incognito/private mode
- User agent override
- Initialization scripts

## Navigation Policy

`NavigationPolicy` supports allow-all, block-all, block-by-scheme, and
allow-by-scheme. The backend emits `NavigationRequested` with the decision.
`OpenExternal` is represented as a decision value for policy evolution; callers
are responsible for opening external URLs.

## Script Evaluation

`evaluate_script` is asynchronous from the caller's perspective. The returned
`ScriptRequestId` appears on a later `WebViewEvent::ScriptResult`. The value is
the serialized result reported by the platform backend.

## IPC

IPC messages include the document URI, body, and a `truncated` flag. The body is
limited by `WebViewOptions::ipc_message_limit` and truncation preserves UTF-8
boundaries.

## Focus Ownership

Native child webviews can participate in OS focus independently from Slint
widgets. Apps should decide when the webview is allowed to accept keyboard
focus, especially on X11/WSLg where pointer location can otherwise affect key
routing. The backend emits `WebViewEvent::FocusRequested` when native pointer
input indicates that the webview should be allowed to claim focus, and
`WebViewEvent::FocusChanged` when native keyboard focus enters or leaves the
webview.

## `WebViewAreaController`

`WebViewAreaController` is the widget-style wrapper for apps that use the
`ui/webview-area.slint` placeholder component. It owns a `WebViewController` and
adds policy for:

- Parking or hiding the native child view when the Slint area is hidden.
- Parking or hiding while Slint modals, menus, or overlays are active.
- Keeping invalid bounds away from the native backend.
- Releasing webview keyboard focus while a Slint shell input owns typing.
- Enabling webview keyboard focus after `WebViewEvent::FocusRequested`.

Important methods:

- `attach`: creates the wrapped native child webview with initial area policy.
- `sync`: applies a `WebViewAreaState` to native bounds, visibility, parking,
  and focus policy.
- `tick`: pumps platform events, calls `sync`, drains events, and applies focus
  event policy.
- `focus_webview`: explicitly lets the webview own keyboard focus.
- `release_keyboard_focus`: disables webview keyboard focus and returns focus to
  the parent window where supported.
- `controller`: exposes the underlying low-level controller for advanced calls.

`WebViewAreaPolicy::default()` uses `HiddenWebViewStrategy::Park`. Hidden or
overlay-covered webviews move to `DEFAULT_PARK_BOUNDS` while remaining natively
visible, which is often faster and less visually stale than the native hide path
on WSLg/WebKitGTK. Apps can choose `Hide` or `HideAndPark` when a platform or
product policy needs a different behavior.

Internally, the root native `WebViewAreaController` delegates to
`slint-webview-core::BackendWebViewAreaController<WebViewController>`.
Non-native backends should reuse the same core area controller by implementing
`WebViewControllerLike` for their public controller type. That keeps overlay,
parking, event-driven focus, and shell-focus release behavior uniform across
native, Servo, and CEF.

## `WebViewBackend`

`slint-webview-core` exposes `WebViewBackend` and
`BackendWebViewController<B>`. Backend crates implement `WebViewBackend` for
their concrete engine surface, then use the shared low-level controller for:

- Event draining.
- Script request ID allocation.
- `WebViewSource` dispatch to blank, URL, or inline HTML.
- Bounds validation before backend updates.
- Shared focus, visibility, and script method shape.

The root native facade already routes its Wry-backed native webview through
`BackendWebViewController<NativeWebView>`. Servo and CEF backends should use the
same shared controller once their concrete engine instances exist.

## `RenderedWebViewBackend`

`RenderedWebViewBackend` is the additional core trait for Slint-owned
composition backends such as Servo and CEF. A rendered backend still implements
`WebViewBackend` for browser operations, then implements
`RenderedWebViewBackend` for:

- Physical-pixel surface resize requests.
- Slint-originated pointer, wheel, keyboard, IME, and focus input events.
- Producing the next available rendered frame.

The shared rendered frame model supports two transports:

- `CpuPixels`: portable paint-buffer output with format and stride metadata.
- `ExternalTexture`: accelerated output identified by an opaque texture ID and
  graphics API.

Servo currently advertises `RenderedWebViewCapabilities::servo_texture()`, which
prefers external textures and keeps CPU pixels as a fallback. CEF advertises
`RenderedWebViewCapabilities::cef_offscreen()`, which prefers CPU pixels first
and leaves room for accelerated textures later.
