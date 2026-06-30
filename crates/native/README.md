# slint-webview-native

Native backend crate for Wry-backed WebView2, WKWebView, and WebKitGTK
integration.

The root `slint-webview` facade depends on this crate for the default native
backend. This crate owns the Wry integration and implements the shared
`slint-webview-core` backend contract.
