use raw_window_handle::HasWindowHandle;

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
    controller: WebViewController,
    policy: WebViewAreaPolicy,
    status: WebViewAreaStatus,
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
        controller.set_keyboard_focus_enabled(keyboard_focus_enabled)?;

        Ok(Self {
            controller,
            policy,
            status: WebViewAreaStatus {
                state,
                placement,
                keyboard_focus_enabled,
            },
        })
    }

    /// Wraps an existing webview controller and immediately applies area policy.
    pub fn from_controller(
        controller: WebViewController,
        state: WebViewAreaState,
        policy: WebViewAreaPolicy,
    ) -> Result<Self> {
        let placement = policy.resolve(state);
        controller.set_bounds(placement.bounds)?;
        controller.set_visible(placement.native_visible)?;
        controller.set_keyboard_focus_enabled(false)?;

        Ok(Self {
            controller,
            policy,
            status: WebViewAreaStatus {
                state,
                placement,
                keyboard_focus_enabled: false,
            },
        })
    }

    /// Returns the capabilities of the selected backend.
    pub fn capabilities() -> WebViewCapabilities {
        WebViewController::capabilities()
    }

    /// Returns the active area policy.
    pub fn policy(&self) -> WebViewAreaPolicy {
        self.policy
    }

    /// Replaces the active area policy and reapplies it to the current state.
    pub fn set_policy(&mut self, policy: WebViewAreaPolicy) -> Result<WebViewAreaStatus> {
        self.policy = policy;
        self.sync(self.status.state)
    }

    /// Returns the latest synchronized status.
    pub fn status(&self) -> WebViewAreaStatus {
        self.status
    }

    /// Returns the underlying low-level controller.
    pub fn controller(&self) -> &WebViewController {
        &self.controller
    }

    /// Synchronizes native bounds, visibility, parking, and focus policy.
    pub fn sync(&mut self, state: WebViewAreaState) -> Result<WebViewAreaStatus> {
        let placement = self.policy.resolve(state);

        if self.status.placement.bounds != placement.bounds {
            self.controller.set_bounds(placement.bounds)?;
        }
        if self.status.placement.native_visible != placement.native_visible {
            self.controller.set_visible(placement.native_visible)?;
        }

        let should_release_focus = (self.policy.release_focus_when_hidden
            && !placement.effective_visible)
            || (self.policy.release_focus_when_shell_focused && state.shell_focus_active);
        let mut keyboard_focus_enabled = self.status.keyboard_focus_enabled;
        if should_release_focus && keyboard_focus_enabled {
            self.release_keyboard_focus()?;
            keyboard_focus_enabled = false;
        }

        self.status = WebViewAreaStatus {
            state,
            placement,
            keyboard_focus_enabled,
        };
        Ok(self.status)
    }

    /// Pumps platform events, synchronizes the area, drains webview events, and
    /// applies built-in focus policy for those events.
    pub fn tick(&mut self, state: WebViewAreaState) -> Result<Vec<WebViewEvent>> {
        pump_platform_events();
        self.sync(state)?;
        self.drain_events()
    }

    /// Drains pending events and applies built-in focus policy for focus events.
    pub fn drain_events(&mut self) -> Result<Vec<WebViewEvent>> {
        let events = self.controller.drain_events();
        for event in &events {
            self.apply_event_policy(event)?;
        }
        Ok(events)
    }

    /// Loads a source into the webview.
    pub fn load_source(&self, source: WebViewSource) -> Result<()> {
        self.controller.load_source(source)
    }

    /// Loads an HTML string into the webview.
    pub fn load_html(&self, html: &str) -> Result<()> {
        self.controller.load_html(html)
    }

    /// Loads a URL into the webview.
    pub fn load_url(&self, url: &str) -> Result<()> {
        self.controller.load_url(url)
    }

    /// Evaluates JavaScript in the webview.
    pub fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId> {
        self.controller.evaluate_script(script)
    }

    /// Allows keyboard focus and requests native focus for the webview when the
    /// area is effectively visible and no shell input owns focus.
    pub fn focus_webview(&mut self) -> Result<()> {
        if !self.status.placement.effective_visible || self.status.state.shell_focus_active {
            self.release_keyboard_focus()?;
            return Ok(());
        }

        self.controller.set_keyboard_focus_enabled(true)?;
        self.controller.focus()?;
        self.status.keyboard_focus_enabled = true;
        Ok(())
    }

    /// Disables webview keyboard focus and returns focus to the parent window
    /// where the backend supports it.
    pub fn release_keyboard_focus(&mut self) -> Result<()> {
        self.controller.set_keyboard_focus_enabled(false)?;
        self.controller.focus_parent()?;
        self.status.keyboard_focus_enabled = false;
        Ok(())
    }

    fn apply_event_policy(&mut self, event: &WebViewEvent) -> Result<()> {
        match event {
            WebViewEvent::FocusRequested => {
                if self.status.placement.effective_visible && !self.status.state.shell_focus_active
                {
                    self.focus_webview()?;
                }
            }
            WebViewEvent::FocusChanged { focused } => {
                self.status.keyboard_focus_enabled = *focused;
                if !focused {
                    self.controller.set_keyboard_focus_enabled(false)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
