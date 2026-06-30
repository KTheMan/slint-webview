# Architecture

`slint-webview` is a workspace with a shared core crate, a root facade crate, a
native backend crate, and backend crate shells for Servo and CEF
implementations.

## Layers

- Core API: option types, events, capabilities, errors, fixtures, area policy,
  `WebViewBackend`, `BackendWebViewController`, `WebViewControllerLike`, and
  `BackendWebViewAreaController` live in `slint-webview-core`.
- Rendered core API: `RenderedWebViewBackend`,
  `BackendRenderedWebViewController`, frame payloads, frame transports,
  rendered surface sizes, dirty rectangles, and Slint-originated input events
  also live in `slint-webview-core`.
- Facade API: the root `slint-webview` crate re-exports core types and provides
  `WebViewController` plus the Tier 1.5 `WebViewAreaController` wrapper.
- Controller: the shared core low-level controller owns the backend instance, event
  receiver, source dispatch, bounds validation, and script request ID
  allocation.
- Area controller: the shared core area controller maps Slint placeholder state
  to backend bounds, visibility, parking, and focus policy.
- Native backend: `slint-webview-native` translates shared backend operations
  to Wry calls.
- Slint component: `ui/webview-area.slint` provides the declarative placeholder
  and callbacks used by applications.
- Regression app: Slint UI plus native webview, compiled only with
  `--features testing`.

## Package Split

The workspace shape is:

- `slint-webview-core`: shared API, event model, area policy, low-level
  controller, area controller, rendered-backend controller/contracts, fixtures,
  docs, and Slint component assets.
- `slint-webview-native`: Wry-backed native platform webview backend.
- `slint-webview-servo`: shell for Servo rendered into Slint-owned textures.
- `slint-webview-cef`: shell for CEF windowless/offscreen Chromium rendered
  into Slint-owned textures.
- `slint-webview`: facade crate that currently selects the native backend by
  default and will later pick alternate backends through features.

The current crate should not expose concrete backend types, because the same
core backend trait and facade-level `WebViewController` /
`WebViewAreaController` contract should be usable by all future backend crates.

## Why Controller First

A controller is honest about the current composition tier. The webview is a
native child view positioned over a Slint window, not a Slint item painted by
the renderer. `WebViewAreaController` is built on top of the core
`BackendWebViewAreaController`, so apps can opt into reusable Slint-facing
policy without exposing Wry or pretending the native child is renderer-owned.

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

Rendered backends implement the normal `WebViewBackend` contract for browser
operations and `RenderedWebViewBackend` for frame production, surface resizing,
and Slint-originated input events. `BackendRenderedWebViewController` combines
those contracts and validates rendered surface sizes and frames before they
reach Slint. The shared frame contract supports CPU pixel buffers as a portable
baseline and external texture identifiers for accelerated paths.

## Error Boundaries

Backend errors are converted to `WebViewError`. Structured errors are used for
disabled backend, missing handles, invalid bounds, blocked navigation, native
operation failures, and platform setup failures.

## Testing Boundary

The fixture page and regression app are not normal library API. They exist to
prove webview composition and host/web messaging without external websites.
