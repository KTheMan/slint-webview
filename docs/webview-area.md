# WebViewArea Composition

`WebViewArea` is the Tier 1.5 composition layer for Slint apps. It pairs a
small Slint placeholder component with `WebViewAreaController`, a Rust wrapper
around `WebViewController`.

This layer makes the native child webview easier to use beside normal Slint
widgets, but it is still not a true scene-graph item. The webview is parked,
hidden, focused, and resized by policy instead of being rendered by Slint.

## Slint Component

The component lives at `ui/webview-area.slint` and can be imported by apps:

```slint
import { WebViewArea } from "path/to/slint-webview/ui/webview-area.slint";

WebViewArea {
    requested-visible: true;
    overlay-active: root.modal-open;
    parked: root.webview-parked;
    status-text: root.webview-status;
    focus-requested => {
        root.focus-webview();
    }
}
```

The component exposes state and callbacks only. It does not create or own the
native webview by itself.

## Rust Controller

Use `WebViewAreaController` when the app wants built-in policy for:

- Applying Slint logical bounds to the native child view.
- Parking the native child offscreen when hidden.
- Suppressing the webview while Slint overlays are active.
- Releasing webview keyboard focus while Slint inputs own typing.
- Enabling webview keyboard focus only after a webview focus request.
- Pumping platform events and draining webview events from one tick call.

```rust,no_run
use slint_webview::{
    WebViewAreaController, WebViewAreaPolicy, WebViewAreaState, WebViewBounds,
    WebViewOptions, WebViewSource,
};

fn attach_area<W>(window: &W) -> slint_webview::Result<WebViewAreaController>
where
    W: raw_window_handle::HasWindowHandle,
{
    let state = WebViewAreaState::new(WebViewBounds::new(276.0, 72.0, 600.0, 504.0));
    let options = WebViewOptions::default()
        .with_source(WebViewSource::Html("<h1>Hello</h1>".to_owned()))
        .with_bounds(state.bounds)
        .with_javascript_enabled(true);

    WebViewAreaController::attach(window, options, state, WebViewAreaPolicy::default())
}
```

Call `sync(state)` whenever Slint area state changes, or call `tick(state)` from
a Slint timer to pump platform events, synchronize policy, and drain events.

## Hidden Strategy

The default hidden strategy is `HiddenWebViewStrategy::Park`. Parking moves the
native child view to `DEFAULT_PARK_BOUNDS` while leaving native visibility on.
This avoids slow or visually stale native hide paths on some platforms,
especially WSLg/WebKitGTK.

Apps can select `Hide` or `HideAndPark` when a platform behaves better with the
native visibility API.

## Focus Contract

The wrapper does not assume hover means keyboard ownership. Slint shell inputs
should set `shell_focus_active = true` while they own typing. The area
controller then disables webview keyboard focus and returns focus to the parent
window where supported.

When the native backend emits `WebViewEvent::FocusRequested`, the area
controller enables webview keyboard focus only if the area is effectively
visible and no shell input is active.
