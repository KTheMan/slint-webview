//! Servo backend crate shell for `slint-webview`.
//!
//! The intended backend target is Slint-owned texture composition with Servo as
//! the web engine.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub use slint_webview_core as core;
pub use slint_webview_core::{
    BackendWebViewAreaController, BackendWebViewController, CompositionTier,
    RenderedWebViewBackend, RenderedWebViewCapabilities, RenderedWebViewDirtyRect,
    RenderedWebViewFrame, RenderedWebViewFrameId, RenderedWebViewFramePayload,
    RenderedWebViewFrameTransport, RenderedWebViewInputEvent, RenderedWebViewInputState,
    RenderedWebViewModifiers, RenderedWebViewPixelFormat, RenderedWebViewPointerButton,
    RenderedWebViewSize, RenderedWebViewTextureApi, WebViewBackend, WebViewCapabilities,
    WebViewControllerLike,
};

/// Returns the planned Servo backend capabilities.
pub const fn planned_capabilities() -> WebViewCapabilities {
    WebViewCapabilities::servo_texture()
}

/// Returns the planned Servo rendered-backend capabilities.
pub const fn planned_rendered_capabilities() -> RenderedWebViewCapabilities {
    RenderedWebViewCapabilities::servo_texture()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn servo_shell_reports_slint_owned_texture_composition() {
        assert_eq!(
            planned_capabilities().composition_tier,
            CompositionTier::SlintOwnedTexture
        );
    }

    #[test]
    fn servo_shell_prefers_external_texture_frames() {
        assert_eq!(
            planned_rendered_capabilities().preferred_transport,
            RenderedWebViewFrameTransport::ExternalTexture
        );
        assert!(planned_rendered_capabilities().supports_cpu_pixels);
    }
}
