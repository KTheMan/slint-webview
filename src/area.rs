use raw_window_handle::HasWindowHandle;
use slint_webview_core::BackendWebViewAreaController;

use crate::{
    Result, ScriptRequestId, WebViewAreaPolicy, WebViewAreaState, WebViewAreaStatus,
    WebViewCapabilities, WebViewController, WebViewEvent, WebViewOptions, WebViewSource,
    pump_platform_events,
};

/// Widget-style controller that keeps a native child webview synchronized with a
/// Slint `WebViewArea` placeholder.
///
/// This is a Tier 1.5 composition layer: it centralizes geometry, parking,
/// overlay, and focus policy, but the underlying webview remains a native child
/// surface rather than a Slint-rendered scene-graph item.
pub struct WebViewAreaController {
    inner: BackendWebViewAreaController<WebViewController>,
}

impl WebViewAreaController {
    /// Creates and attaches a native webview using area policy for its initial
    /// bounds, native visibility, and keyboard focusability.
    pub fn attach<W>(
        window: &W,
        mut options: WebViewOptions,
        state: WebViewAreaState,
        policy: WebViewAreaPolicy,
    ) -> Result<Self>
    where
        W: HasWindowHandle,
    {
        let placement = policy.resolve(state);
        let keyboard_focus_enabled =
            options.focused && placement.effective_visible && !state.shell_focus_active;
        options.bounds = placement.bounds;
        options.visible = placement.native_visible;
        options.focused = keyboard_focus_enabled;

        let controller = WebViewController::attach(window, options)?;
        let inner = BackendWebViewAreaController::from_controller_with_keyboard_focus(
            controller,
            state,
            policy,
            keyboard_focus_enabled,
        )?;

        Ok(Self { inner })
    }

    /// Wraps an existing webview controller and immediately applies area policy.
    pub fn from_controller(
        controller: WebViewController,
        state: WebViewAreaState,
        policy: WebViewAreaPolicy,
    ) -> Result<Self> {
        Ok(Self {
            inner: BackendWebViewAreaController::from_controller(controller, state, policy)?,
        })
    }

    /// Returns the capabilities of the selected backend.
    pub fn capabilities() -> WebViewCapabilities {
        BackendWebViewAreaController::<WebViewController>::capabilities()
    }

    /// Returns the active area policy.
    pub fn policy(&self) -> WebViewAreaPolicy {
        self.inner.policy()
    }

    /// Replaces the active area policy and reapplies it to the current state.
    pub fn set_policy(&mut self, policy: WebViewAreaPolicy) -> Result<WebViewAreaStatus> {
        self.inner.set_policy(policy)
    }

    /// Returns the latest synchronized status.
    pub fn status(&self) -> WebViewAreaStatus {
        self.inner.status()
    }

    /// Returns the underlying low-level controller.
    pub fn controller(&self) -> &WebViewController {
        self.inner.controller()
    }

    /// Synchronizes native bounds, visibility, parking, and focus policy.
    pub fn sync(&mut self, state: WebViewAreaState) -> Result<WebViewAreaStatus> {
        self.inner.sync(state)
    }

    /// Pumps platform events, synchronizes the area, drains webview events, and
    /// applies built-in focus policy for those events.
    pub fn tick(&mut self, state: WebViewAreaState) -> Result<Vec<WebViewEvent>> {
        self.inner.tick_with(state, pump_platform_events)
    }

    /// Drains pending events and applies built-in focus policy for focus events.
    pub fn drain_events(&mut self) -> Result<Vec<WebViewEvent>> {
        self.inner.drain_events()
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

    /// Evaluates JavaScript in the webview.
    pub fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId> {
        self.inner.evaluate_script(script)
    }

    /// Allows keyboard focus and requests native focus for the webview when the
    /// area is effectively visible and no shell input owns focus.
    pub fn focus_webview(&mut self) -> Result<()> {
        self.inner.focus_webview()
    }

    /// Disables webview keyboard focus and returns focus to the parent window
    /// where the backend supports it.
    pub fn release_keyboard_focus(&mut self) -> Result<()> {
        self.inner.release_keyboard_focus()
    }
}
