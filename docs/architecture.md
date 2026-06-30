# Architecture

`slint-webview` is split into a small public API and a private backend adapter.

## Layers

- Public API: option types, events, capabilities, errors, and
  `WebViewController`.
- Controller: owns the backend instance, event receiver, and script request ID
  allocation.
- Backend adapter: translates controller operations to Wry calls.
- Regression app: Slint UI plus native webview, compiled only with
  `--features testing`.

## Why Controller First

A controller is honest about the current composition tier. The webview is a
native child view positioned over a Slint window, not a Slint item painted by
the renderer. A future Slint component wrapper can be built on top of the
controller, but the controller remains the portable ownership and event model.

## Native Child Composition

Native child views are efficient and use the platform browser runtime, but they
do not inherit all Slint scene-graph behavior. The controller exposes bounds and
visibility so application code can synchronize native geometry with Slint
layout.

Known policy choices:

- Hide the webview while Slint modals or menus should appear above it.
- Avoid relying on Slint clipping or transforms for the webview area.
- Treat resize and scale updates as app responsibilities.

## Error Boundaries

Backend errors are converted to `WebViewError`. Structured errors are used for
disabled backend, missing handles, invalid bounds, blocked navigation, native
operation failures, and platform setup failures.

## Testing Boundary

The fixture page and regression app are not normal library API. They exist to
prove webview composition and host/web messaging without external websites.
