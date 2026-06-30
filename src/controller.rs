use std::cell::Cell;
use std::sync::mpsc::{self, Receiver};

use raw_window_handle::HasWindowHandle;

use crate::platform;
use crate::{
    Result, ScriptRequestId, WebViewBounds, WebViewCapabilities, WebViewError, WebViewEvent,
    WebViewOptions, WebViewSource,
};

/// Primary controller for an embedded native webview.
///
/// The controller owns the platform webview instance and exposes a portable
/// surface for loading content, resizing, focusing, evaluating JavaScript, and
/// receiving webview events.
pub struct WebViewController {
    backend: platform::NativeWebView,
    events: Receiver<WebViewEvent>,
    next_script_request_id: Cell<u64>,
}

impl WebViewController {
    /// Creates and attaches a native child webview to a Slint window handle.
    pub fn attach<W>(window: &W, options: WebViewOptions) -> Result<Self>
    where
        W: HasWindowHandle,
    {
        validate_bounds(options.bounds)?;

        let (sender, events) = mpsc::channel();
        let backend = platform::NativeWebView::attach(window, options, sender)?;

        Ok(Self {
            backend,
            events,
            next_script_request_id: Cell::new(1),
        })
    }

    /// Returns the capabilities of the selected backend.
    pub fn capabilities() -> WebViewCapabilities {
        platform::NativeWebView::capabilities()
    }

    /// Attempts to receive one pending event.
    pub fn try_recv_event(&self) -> Option<WebViewEvent> {
        self.events.try_recv().ok()
    }

    /// Drains all events currently queued for this controller.
    pub fn drain_events(&self) -> Vec<WebViewEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.try_recv_event() {
            events.push(event);
        }
        events
    }

    /// Updates the webview bounds in Slint logical window coordinates.
    pub fn set_bounds(&self, bounds: WebViewBounds) -> Result<()> {
        validate_bounds(bounds)?;
        self.backend.set_bounds(bounds)
    }

    /// Shows or hides the native webview.
    pub fn set_visible(&self, visible: bool) -> Result<()> {
        self.backend.set_visible(visible)
    }

    /// Loads a source into the webview.
    pub fn load_source(&self, source: WebViewSource) -> Result<()> {
        match source {
            WebViewSource::Blank => self.load_html(""),
            WebViewSource::Url(url) => self.load_url(&url),
            WebViewSource::Html(html) => self.load_html(&html),
        }
    }

    /// Loads an HTML string into the webview.
    pub fn load_html(&self, html: &str) -> Result<()> {
        self.backend.load_html(html)
    }

    /// Loads a URL into the webview.
    pub fn load_url(&self, url: &str) -> Result<()> {
        self.backend.load_url(url)
    }

    /// Evaluates JavaScript and returns the request ID that will appear on the
    /// matching [`WebViewEvent::ScriptResult`] event.
    pub fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId> {
        let request_id = self.allocate_script_request_id();
        self.backend.evaluate_script(script, request_id)?;
        Ok(request_id)
    }

    /// Requests focus for the native webview.
    pub fn focus(&self) -> Result<()> {
        self.backend.focus()
    }

    /// Enables or disables whether the native webview can take keyboard focus.
    ///
    /// This is useful when another native or Slint text input is active and a
    /// platform child surface would otherwise steal keyboard focus on hover.
    /// Platforms without a direct focusability control treat this as a no-op.
    pub fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
        self.backend.set_keyboard_focus_enabled(enabled)
    }

    /// Returns focus to the native parent window where supported.
    pub fn focus_parent(&self) -> Result<()> {
        self.backend.focus_parent()
    }

    fn allocate_script_request_id(&self) -> ScriptRequestId {
        let id = self.next_script_request_id.get();
        self.next_script_request_id.set(id.saturating_add(1).max(1));
        ScriptRequestId(id)
    }
}

/// Initializes platform-specific webview prerequisites.
pub fn initialize_platform() -> Result<()> {
    platform::initialize_platform()
}

/// Pumps platform-specific webview events that are not driven by Slint.
pub fn pump_platform_events() {
    platform::pump_platform_events();
}

fn validate_bounds(bounds: WebViewBounds) -> Result<()> {
    if bounds.is_valid() {
        Ok(())
    } else {
        Err(WebViewError::InvalidBounds(bounds))
    }
}
