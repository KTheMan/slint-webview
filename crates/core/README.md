# slint-webview-core

Shared API, capability, error, fixture, and area-controller types for the
`slint-webview` backend family.

This crate intentionally contains no browser engine. Backend crates such as
`slint-webview-native`, `slint-webview-servo`, and `slint-webview-cef` should
depend on it and implement only the final rendering/runtime integration layer.

Backend crates implement `WebViewBackend` for their concrete engine surface and
use `BackendWebViewController` for common event draining, script request IDs,
source dispatch, and bounds validation.

Backend crates can also implement `WebViewControllerLike` for their public
controller type and reuse `BackendWebViewAreaController` for Slint placeholder
sync, hiding/parking policy, and focus-event handling.
