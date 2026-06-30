//! CEF backend crate shell for `slint-webview`.
//!
//! The intended backend target is Slint-owned texture or paint-buffer
//! composition with Chromium/CEF as the web engine.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub use slint_webview_core as core;
pub use slint_webview_core::{
    BackendWebViewAreaController, BackendWebViewController, CompositionTier, WebViewBackend,
    WebViewCapabilities, WebViewControllerLike,
};

/// Returns the planned CEF backend capabilities.
pub const fn planned_capabilities() -> WebViewCapabilities {
    WebViewCapabilities::cef_offscreen()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cef_shell_reports_slint_owned_texture_composition() {
        assert_eq!(
            planned_capabilities().composition_tier,
            CompositionTier::SlintOwnedTexture
        );
    }
}
