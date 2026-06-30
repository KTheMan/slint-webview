# slint-webview-cef

Planned CEF backend crate for windowless/offscreen Chromium composition.

This crate is intentionally a shell until the CEF subprocess, paint buffer,
accelerated texture, and packaging implementation lands behind the shared core
API.

The crate currently re-exports the shared `RenderedWebViewBackend` contract and
reports planned rendered capabilities that prefer CPU paint-buffer frames while
allowing accelerated texture paths later.
