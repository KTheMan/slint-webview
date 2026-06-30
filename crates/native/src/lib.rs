//! Native platform backend crate shell for `slint-webview`.
//!
//! This crate currently exposes the shared core and native capability metadata.
//! The Wry implementation still lives in the root facade crate until the backend
//! split is completed.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub use slint_webview_core as core;
pub use slint_webview_core::{CompositionTier, WebViewCapabilities};

/// Returns the planned native backend capabilities.
pub const fn planned_capabilities() -> WebViewCapabilities {
    WebViewCapabilities::wry_native()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_shell_reports_native_child_composition() {
        assert_eq!(
            planned_capabilities().composition_tier,
            CompositionTier::NativeChildView
        );
    }
}
