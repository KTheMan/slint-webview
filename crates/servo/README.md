# slint-webview-servo

Planned Servo backend crate for Slint-owned texture composition.

This crate is intentionally a shell until the Servo event loop, texture sharing,
and input translation implementation lands behind the shared core API.

The crate currently re-exports the shared `RenderedWebViewBackend` contract and
reports planned rendered capabilities that prefer external texture frames while
allowing CPU pixel fallback.
