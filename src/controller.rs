use std::sync::mpsc;

use raw_window_handle::HasWindowHandle;
use slint_webview_core::{BackendWebViewController, WebViewControllerLike, validate_bounds};
use slint_webview_native::{
    NativeWebView, initialize_platform as initialize_native_platform,
    pump_platform_events as pump_native_platform_events,
};

use crate::{
    Result, ScriptRequestId, WebViewBounds, WebViewCapabilities, WebViewEvent, WebViewOptions,
    WebViewSource,
};

/// Primary controller for an embedded native webview.
///
/// The controller owns the platform webview instance and exposes a portable
/// surface for loading content, resizing, focusing, evaluating JavaScript, and
/// receiving webview events.
pub struct WebViewController {
    inner: BackendWebViewController<NativeWebView>,
}

impl WebViewController {
    /// Creates and attaches a native child webview to a Slint window handle.
    pub fn attach<W>(window: &W, options: WebViewOptions) -> Result<Self>
    where
        W: HasWindowHandle,
    {
        validate_bounds(options.bounds)?;

        let (sender, events) = mpsc::channel();
        let backend = NativeWebView::attach(window, options, sender)?;
        let inner = BackendWebViewController::new(backend, events);

        Ok(Self { inner })
    }

    /// Returns the capabilities of the selected backend.
    pub fn capabilities() -> WebViewCapabilities {
        BackendWebViewController::<NativeWebView>::capabilities()
    }

    /// Attempts to receive one pending event.
    pub fn try_recv_event(&self) -> Option<WebViewEvent> {
        self.inner.try_recv_event()
    }

    /// Drains all events currently queued for this controller.
    pub fn drain_events(&self) -> Vec<WebViewEvent> {
        self.inner.drain_events()
    }

    /// Updates the webview bounds in Slint logical window coordinates.
    pub fn set_bounds(&self, bounds: WebViewBounds) -> Result<()> {
        self.inner.set_bounds(bounds)
    }

    /// Shows or hides the native webview.
    pub fn set_visible(&self, visible: bool) -> Result<()> {
        self.inner.set_visible(visible)
    }

    /// Loads a source into the webview.
    pub fn load_source(&self, source: WebViewSource) -> Result<()> {
        self.inner.load_source(source)
    }

    /// Loads an HTML string into the webview.
    pub fn load_html(&self, html: &str) -> Result<()> {
        self.inner.load_html(html)
    }

    /// Loads a URL into the webview.
    pub fn load_url(&self, url: &str) -> Result<()> {
        self.inner.load_url(url)
    }

    /// Evaluates JavaScript and returns the request ID that will appear on the
    /// matching [`WebViewEvent::ScriptResult`] event.
    pub fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId> {
        self.inner.evaluate_script(script)
    }

    /// Requests focus for the native webview.
    pub fn focus(&self) -> Result<()> {
        self.inner.focus()
    }

    /// Enables or disables whether the native webview can take keyboard focus.
    ///
    /// This is useful when another native or Slint text input is active and a
    /// platform child surface would otherwise steal keyboard focus on hover.
    /// Platforms without a direct focusability control treat this as a no-op.
    pub fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
        self.inner.set_keyboard_focus_enabled(enabled)
    }

    /// Returns focus to the native parent window where supported.
    pub fn focus_parent(&self) -> Result<()> {
        self.inner.focus_parent()
    }
}

/// Initializes platform-specific webview prerequisites.
pub fn initialize_platform() -> Result<()> {
    initialize_native_platform()
}

/// Pumps platform-specific webview events that are not driven by Slint.
pub fn pump_platform_events() {
    pump_native_platform_events();
}

impl WebViewControllerLike for WebViewController {
    fn capabilities() -> WebViewCapabilities {
        WebViewController::capabilities()
    }

    fn drain_events(&self) -> Vec<WebViewEvent> {
        WebViewController::drain_events(self)
    }

    fn set_bounds(&self, bounds: WebViewBounds) -> Result<()> {
        WebViewController::set_bounds(self, bounds)
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        WebViewController::set_visible(self, visible)
    }

    fn load_source(&self, source: WebViewSource) -> Result<()> {
        WebViewController::load_source(self, source)
    }

    fn load_html(&self, html: &str) -> Result<()> {
        WebViewController::load_html(self, html)
    }

    fn load_url(&self, url: &str) -> Result<()> {
        WebViewController::load_url(self, url)
    }

    fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId> {
        WebViewController::evaluate_script(self, script)
    }

    fn focus(&self) -> Result<()> {
        WebViewController::focus(self)
    }

    fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
        WebViewController::set_keyboard_focus_enabled(self, enabled)
    }

    fn focus_parent(&self) -> Result<()> {
        WebViewController::focus_parent(self)
    }
}
