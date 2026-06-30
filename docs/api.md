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
