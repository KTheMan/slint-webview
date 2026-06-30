# slint-webview-core

Shared API, capability, error, fixture, and area-policy types for the
`slint-webview` backend family.

This crate intentionally contains no browser engine. Backend crates such as
`slint-webview-native`, `slint-webview-servo`, and `slint-webview-cef` should
depend on it and implement only the final rendering/runtime integration layer.
