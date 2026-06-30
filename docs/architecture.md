# Architecture

`slint-webview` is currently split into a small public API and a private Wry
backend adapter. It is intended to evolve into a shared-core package plus
backend-specific crates.

## Layers

- Public API: option types, events, capabilities, errors, `WebViewController`,
  and the Tier 1.5 `WebViewAreaController` policy wrapper.
- Controller: owns the backend instance, event receiver, and script request ID
  allocation.
- Area controller: maps Slint placeholder state to native bounds, visibility,
  parking, and focus policy.
- Backend adapter: translates controller operations to Wry calls.
- Slint component: `ui/webview-area.slint` provides the declarative placeholder
  and callbacks used by applications.
- Regression app: Slint UI plus native webview, compiled only with
  `--features testing`.

## Future Package Split

The preferred long-term shape is:

- `slint-webview-core`: shared API, event model, area policy, fixtures, docs,
  and Slint component assets.
- `slint-webview-native`: Wry and native platform webviews.
- `slint-webview-servo`: Servo rendered into Slint-owned textures.
- `slint-webview-cef`: CEF windowless/offscreen Chromium rendered into
  Slint-owned textures.
- `slint-webview`: facade crate that picks a backend through features.

The current crate should not expose backend types, because the same
`WebViewController` and `WebViewAreaController` contract should be usable by all
future backend crates.

## Why Controller First

A controller is honest about the current composition tier. The webview is a
native child view positioned over a Slint window, not a Slint item painted by
the renderer. `WebViewAreaController` is built on top of the low-level
controller, so apps can opt into reusable Slint-facing policy without exposing
Wry or pretending the native child is renderer-owned.

## Native Child Composition

Native child views are efficient and use the platform browser runtime, but they
do not inherit all Slint scene-graph behavior. The controller exposes bounds and
visibility so application code can synchronize native geometry with Slint
layout.

Known policy choices:

- Park or hide the webview while Slint modals or menus should appear above it.
- Avoid relying on Slint clipping or transforms for the webview area.
- Treat resize and scale updates as app responsibilities unless they are routed
  through `WebViewAreaController::sync`.

## Texture Composition

Servo and CEF backends should target `CompositionTier::SlintOwnedTexture`. In
that model, Slint receives a texture or pixel buffer from the backend and owns
z-order, clipping, overlays, and visual composition. The backend still owns web
layout, JavaScript, network loading, storage, and browser-process policy.

## Error Boundaries

Backend errors are converted to `WebViewError`. Structured errors are used for
disabled backend, missing handles, invalid bounds, blocked navigation, native
operation failures, and platform setup failures.

## Testing Boundary

The fixture page and regression app are not normal library API. They exist to
prove webview composition and host/web messaging without external websites.
