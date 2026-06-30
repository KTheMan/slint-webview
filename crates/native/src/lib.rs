//! Native platform backend crate for `slint-webview`.
//!
//! This crate owns the Wry-backed native child-view implementation. The root
//! `slint-webview` crate is a facade over this backend plus the shared core API.

#![deny(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "backend-wry")]
mod wry_backend;

pub use slint_webview_core as core;
pub use slint_webview_core::{
    BackendWebViewAreaController, BackendWebViewController, CompositionTier, Result,
    ScriptRequestId, WebViewBackend, WebViewBounds, WebViewCapabilities, WebViewControllerLike,
    WebViewError, WebViewEvent, WebViewOptions,
};

#[cfg(feature = "backend-wry")]
pub use wry_backend::{NativeWebView, initialize_platform, pump_platform_events};

#[cfg(not(feature = "backend-wry"))]
/// Disabled native backend used when `backend-wry` is not enabled.
pub struct NativeWebView;

#[cfg(not(feature = "backend-wry"))]
impl NativeWebView {
    /// Attempts to attach the native backend.
    pub fn attach<W>(
        _window: &W,
        _options: WebViewOptions,
        _event_sender: std::sync::mpsc::Sender<WebViewEvent>,
    ) -> Result<Self>
    where
        W: raw_window_handle::HasWindowHandle,
    {
        Err(WebViewError::BackendDisabled)
    }

    /// Returns unsupported capabilities for the disabled backend.
    pub fn capabilities() -> WebViewCapabilities {
        WebViewCapabilities::unsupported()
    }

    /// Updates webview bounds.
    pub fn set_bounds(&self, _bounds: WebViewBounds) -> Result<()> {
        Err(WebViewError::BackendDisabled)
    }

    /// Shows or hides the native webview.
    pub fn set_visible(&self, _visible: bool) -> Result<()> {
        Err(WebViewError::BackendDisabled)
    }

    /// Loads an HTML string.
    pub fn load_html(&self, _html: &str) -> Result<()> {
        Err(WebViewError::BackendDisabled)
    }

    /// Loads a URL.
    pub fn load_url(&self, _url: &str) -> Result<()> {
        Err(WebViewError::BackendDisabled)
    }

    /// Evaluates JavaScript.
    pub fn evaluate_script(&self, _script: &str, _request_id: ScriptRequestId) -> Result<()> {
        Err(WebViewError::BackendDisabled)
    }

    /// Requests focus for the native webview.
    pub fn focus(&self) -> Result<()> {
        Err(WebViewError::BackendDisabled)
    }

    /// Enables or disables native keyboard focus.
    pub fn set_keyboard_focus_enabled(&self, _enabled: bool) -> Result<()> {
        Err(WebViewError::BackendDisabled)
    }

    /// Returns focus to the native parent window.
    pub fn focus_parent(&self) -> Result<()> {
        Err(WebViewError::BackendDisabled)
    }
}

#[cfg(not(feature = "backend-wry"))]
/// Initializes platform-specific native webview prerequisites.
pub fn initialize_platform() -> Result<()> {
    Err(WebViewError::BackendDisabled)
}

#[cfg(not(feature = "backend-wry"))]
/// Pumps platform-specific native webview events.
pub fn pump_platform_events() {}

impl WebViewBackend for NativeWebView {
    fn capabilities() -> WebViewCapabilities {
        NativeWebView::capabilities()
    }

    fn set_bounds(&self, bounds: WebViewBounds) -> Result<()> {
        NativeWebView::set_bounds(self, bounds)
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        NativeWebView::set_visible(self, visible)
    }

    fn load_html(&self, html: &str) -> Result<()> {
        NativeWebView::load_html(self, html)
    }

    fn load_url(&self, url: &str) -> Result<()> {
        NativeWebView::load_url(self, url)
    }

    fn evaluate_script(&self, script: &str, request_id: ScriptRequestId) -> Result<()> {
        NativeWebView::evaluate_script(self, script, request_id)
    }

    fn focus(&self) -> Result<()> {
        NativeWebView::focus(self)
    }

    fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
        NativeWebView::set_keyboard_focus_enabled(self, enabled)
    }

    fn focus_parent(&self) -> Result<()> {
        NativeWebView::focus_parent(self)
    }
}

/// Returns the native backend capabilities.
pub fn planned_capabilities() -> WebViewCapabilities {
    NativeWebView::capabilities()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "backend-wry")]
    fn native_backend_reports_native_child_composition() {
        assert_eq!(
            planned_capabilities().composition_tier,
            CompositionTier::NativeChildView
        );
    }

    #[test]
    #[cfg(not(feature = "backend-wry"))]
    fn disabled_native_backend_reports_unsupported_composition() {
        assert_eq!(
            planned_capabilities().composition_tier,
            CompositionTier::Unsupported
        );
    }
}
