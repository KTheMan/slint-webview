//! CEF backend crate shell for `slint-webview`.
//!
//! The intended backend target is Slint-owned texture or paint-buffer
//! composition with Chromium/CEF as the web engine.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub use slint_webview_core as core;
pub use slint_webview_core::{
    BackendRenderedWebViewController, BackendWebViewAreaController, BackendWebViewController,
    CompositionTier, RenderedWebViewBackend, RenderedWebViewCapabilities, RenderedWebViewDirtyRect,
    RenderedWebViewFrame, RenderedWebViewFrameId, RenderedWebViewFramePayload,
    RenderedWebViewFrameTransport, RenderedWebViewInputEvent, RenderedWebViewInputState,
    RenderedWebViewModifiers, RenderedWebViewPixelFormat, RenderedWebViewPointerButton,
    RenderedWebViewSize, RenderedWebViewTextureApi, WebViewBackend, WebViewCapabilities,
    WebViewControllerLike, validate_rendered_frame, validate_rendered_size,
};

/// Returns the planned CEF backend capabilities.
pub const fn planned_capabilities() -> WebViewCapabilities {
    WebViewCapabilities::cef_offscreen()
}

/// Returns the planned CEF rendered-backend capabilities.
pub const fn planned_rendered_capabilities() -> RenderedWebViewCapabilities {
    RenderedWebViewCapabilities::cef_offscreen()
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

    #[test]
    fn cef_shell_prefers_cpu_pixel_frames() {
        assert_eq!(
            planned_rendered_capabilities().preferred_transport,
            RenderedWebViewFrameTransport::CpuPixels
        );
        assert!(planned_rendered_capabilities().supports_external_textures);
    }
}
