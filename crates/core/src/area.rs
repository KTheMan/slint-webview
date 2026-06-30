use serde::{Deserialize, Serialize};

use crate::{
    BackendWebViewController, Result, ScriptRequestId, WebViewBackend, WebViewBounds,
    WebViewCapabilities, WebViewEvent, WebViewSource,
};

/// Default offscreen rectangle used when a native child webview is parked.
pub const DEFAULT_PARK_BOUNDS: WebViewBounds = WebViewBounds::new(-32000.0, -32000.0, 1.0, 1.0);

/// Strategy used when the Slint area should not expose the webview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HiddenWebViewStrategy {
    /// Keep the last real bounds but call the native visibility API.
    Hide,
    /// Move the webview surface offscreen while leaving native visibility on.
    Park,
    /// Move the webview surface offscreen and call the native visibility API.
    HideAndPark,
}

impl HiddenWebViewStrategy {
    /// Returns true when the hidden strategy moves the webview offscreen.
    pub const fn parks(self) -> bool {
        matches!(self, Self::Park | Self::HideAndPark)
    }

    /// Returns the native visibility flag to use while the area is hidden.
    pub const fn native_visible_when_hidden(self) -> bool {
        matches!(self, Self::Park)
    }
}

/// Policy for mapping a Slint placeholder area to backend operations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WebViewAreaPolicy {
    /// Hides the webview whenever a Slint overlay, modal, menu, or tooltip
    /// needs to draw above the area.
    pub hide_when_overlay_active: bool,
    /// Hides the webview until the area reports finite positive bounds.
    pub hide_until_bounds_valid: bool,
    /// Operation used when the area is hidden.
    pub hidden_strategy: HiddenWebViewStrategy,
    /// Offscreen bounds used by parking strategies.
    pub park_bounds: WebViewBounds,
    /// Releases webview keyboard ownership when the webview is hidden.
    pub release_focus_when_hidden: bool,
    /// Releases webview keyboard ownership when a Slint input owns shell focus.
    pub release_focus_when_shell_focused: bool,
}

impl Default for WebViewAreaPolicy {
    fn default() -> Self {
        Self {
            hide_when_overlay_active: true,
            hide_until_bounds_valid: true,
            hidden_strategy: HiddenWebViewStrategy::Park,
            park_bounds: DEFAULT_PARK_BOUNDS,
            release_focus_when_hidden: true,
            release_focus_when_shell_focused: true,
        }
    }
}

impl WebViewAreaPolicy {
    /// Returns a policy with a different hidden webview strategy.
    pub fn with_hidden_strategy(mut self, strategy: HiddenWebViewStrategy) -> Self {
        self.hidden_strategy = strategy;
        self
    }

    /// Returns a policy with different parking bounds.
    pub fn with_park_bounds(mut self, bounds: WebViewBounds) -> Self {
        self.park_bounds = bounds;
        self
    }

    /// Returns a policy with overlay-driven hiding enabled or disabled.
    pub fn with_hide_when_overlay_active(mut self, enabled: bool) -> Self {
        self.hide_when_overlay_active = enabled;
        self
    }

    /// Returns a policy with invalid-bounds hiding enabled or disabled.
    pub fn with_hide_until_bounds_valid(mut self, enabled: bool) -> Self {
        self.hide_until_bounds_valid = enabled;
        self
    }

    /// Resolves Slint-side state into backend placement.
    pub fn resolve(self, state: WebViewAreaState) -> WebViewAreaPlacement {
        let bounds_valid = state.bounds.is_valid();
        let blocked_by_overlay = self.hide_when_overlay_active && state.overlay_active;
        let blocked_by_invalid_bounds = self.hide_until_bounds_valid && !bounds_valid;
        let effective_visible =
            state.requested_visible && !blocked_by_overlay && !blocked_by_invalid_bounds;
        let parked = !effective_visible && self.hidden_strategy.parks();
        let native_visible = effective_visible || self.hidden_strategy.native_visible_when_hidden();
        let fallback_bounds = self.valid_park_bounds();
        let bounds = if parked || !bounds_valid {
            fallback_bounds
        } else {
            state.bounds
        };

        WebViewAreaPlacement {
            bounds,
            native_visible,
            effective_visible,
            parked,
            blocked_by_overlay,
            blocked_by_invalid_bounds,
        }
    }

    fn valid_park_bounds(self) -> WebViewBounds {
        if self.park_bounds.is_valid() {
            self.park_bounds
        } else {
            DEFAULT_PARK_BOUNDS
        }
    }
}

/// Slint-side state for a webview placeholder area.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WebViewAreaState {
    /// Placeholder bounds in Slint logical window coordinates.
    pub bounds: WebViewBounds,
    /// Whether the application wants the webview exposed.
    pub requested_visible: bool,
    /// Whether Slint UI above the webview needs z-order priority.
    pub overlay_active: bool,
    /// Whether a Slint/native shell input currently owns keyboard typing.
    pub shell_focus_active: bool,
}

impl Default for WebViewAreaState {
    fn default() -> Self {
        Self {
            bounds: WebViewBounds::default(),
            requested_visible: true,
            overlay_active: false,
            shell_focus_active: false,
        }
    }
}

impl WebViewAreaState {
    /// Creates state for a visible webview area with no overlay or shell focus.
    pub const fn new(bounds: WebViewBounds) -> Self {
        Self {
            bounds,
            requested_visible: true,
            overlay_active: false,
            shell_focus_active: false,
        }
    }

    /// Returns state with different bounds.
    pub fn with_bounds(mut self, bounds: WebViewBounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// Returns state with requested visibility changed.
    pub fn with_requested_visible(mut self, visible: bool) -> Self {
        self.requested_visible = visible;
        self
    }

    /// Returns state with overlay activity changed.
    pub fn with_overlay_active(mut self, active: bool) -> Self {
        self.overlay_active = active;
        self
    }

    /// Returns state with shell focus activity changed.
    pub fn with_shell_focus_active(mut self, active: bool) -> Self {
        self.shell_focus_active = active;
        self
    }
}

/// Resolved backend placement for a webview area.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WebViewAreaPlacement {
    /// Bounds that should be applied to the backend.
    pub bounds: WebViewBounds,
    /// Native visibility flag that should be applied to child-view backends.
    pub native_visible: bool,
    /// True when the webview should be visible inside the Slint area.
    pub effective_visible: bool,
    /// True when the webview should be moved offscreen.
    pub parked: bool,
    /// True when an active Slint overlay suppressed the webview.
    pub blocked_by_overlay: bool,
    /// True when invalid area bounds suppressed the webview.
    pub blocked_by_invalid_bounds: bool,
}

/// Current status reported by a webview area controller.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WebViewAreaStatus {
    /// Last Slint-side area state passed to the controller.
    pub state: WebViewAreaState,
    /// Last resolved backend placement.
    pub placement: WebViewAreaPlacement,
    /// True when the backend may accept keyboard focus.
    pub keyboard_focus_enabled: bool,
}

/// Portable controller operations required by [`BackendWebViewAreaController`].
///
/// Backend crates can implement this trait directly for their public
/// controller type. The shared area controller then owns the policy for
/// geometry, visibility, focus release, and focus-request events without
/// knowing whether the underlying engine is native, Servo, or CEF.
pub trait WebViewControllerLike {
    /// Returns static capabilities for this controller backend.
    fn capabilities() -> WebViewCapabilities
    where
        Self: Sized;

    /// Drains all events currently queued for this controller.
    fn drain_events(&self) -> Vec<WebViewEvent>;

    /// Updates webview bounds in Slint logical window coordinates.
    fn set_bounds(&self, bounds: WebViewBounds) -> Result<()>;

    /// Shows or hides the backend surface.
    fn set_visible(&self, visible: bool) -> Result<()>;

    /// Loads a source into the webview.
    fn load_source(&self, source: WebViewSource) -> Result<()>;

    /// Loads an HTML string into the webview.
    fn load_html(&self, html: &str) -> Result<()>;

    /// Loads a URL into the webview.
    fn load_url(&self, url: &str) -> Result<()>;

    /// Evaluates JavaScript and returns the matching request ID.
    fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId>;

    /// Requests keyboard focus for the backend.
    fn focus(&self) -> Result<()>;

    /// Enables or disables whether the backend may take keyboard focus.
    fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()>;

    /// Returns focus to the host shell where supported.
    fn focus_parent(&self) -> Result<()>;
}

impl<B> WebViewControllerLike for BackendWebViewController<B>
where
    B: WebViewBackend,
{
    fn capabilities() -> WebViewCapabilities {
        BackendWebViewController::<B>::capabilities()
    }

    fn drain_events(&self) -> Vec<WebViewEvent> {
        BackendWebViewController::drain_events(self)
    }

    fn set_bounds(&self, bounds: WebViewBounds) -> Result<()> {
        BackendWebViewController::set_bounds(self, bounds)
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        BackendWebViewController::set_visible(self, visible)
    }

    fn load_source(&self, source: WebViewSource) -> Result<()> {
        BackendWebViewController::load_source(self, source)
    }

    fn load_html(&self, html: &str) -> Result<()> {
        BackendWebViewController::load_html(self, html)
    }

    fn load_url(&self, url: &str) -> Result<()> {
        BackendWebViewController::load_url(self, url)
    }

    fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId> {
        BackendWebViewController::evaluate_script(self, script)
    }

    fn focus(&self) -> Result<()> {
        BackendWebViewController::focus(self)
    }

    fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
        BackendWebViewController::set_keyboard_focus_enabled(self, enabled)
    }

    fn focus_parent(&self) -> Result<()> {
        BackendWebViewController::focus_parent(self)
    }
}

/// Shared area controller for a backend-specific webview controller.
///
/// This is the backend-agnostic composition layer for a Slint `WebViewArea`
/// placeholder. It synchronizes native or rendered webview placement with the
/// Slint-side area state, hides or parks the webview behind Slint overlays, and
/// applies shared keyboard-focus policy.
pub struct BackendWebViewAreaController<C> {
    controller: C,
    policy: WebViewAreaPolicy,
    status: WebViewAreaStatus,
}

impl<C> BackendWebViewAreaController<C>
where
    C: WebViewControllerLike,
{
    /// Wraps an existing controller and applies area policy with keyboard focus
    /// disabled.
    pub fn from_controller(
        controller: C,
        state: WebViewAreaState,
        policy: WebViewAreaPolicy,
    ) -> Result<Self> {
        Self::from_controller_with_keyboard_focus(controller, state, policy, false)
    }

    /// Wraps an existing controller and applies area policy with optional
    /// initial keyboard focus.
    ///
    /// Initial keyboard focus is only retained when the resolved placement is
    /// effectively visible and no shell input owns focus.
    pub fn from_controller_with_keyboard_focus(
        controller: C,
        state: WebViewAreaState,
        policy: WebViewAreaPolicy,
        keyboard_focus_enabled: bool,
    ) -> Result<Self> {
        let placement = policy.resolve(state);
        let keyboard_focus_enabled =
            keyboard_focus_enabled && placement.effective_visible && !state.shell_focus_active;
        controller.set_bounds(placement.bounds)?;
        controller.set_visible(placement.native_visible)?;
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

    /// Returns the capabilities of the selected backend.
    pub fn capabilities() -> WebViewCapabilities {
        C::capabilities()
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

    /// Returns the wrapped backend-specific controller.
    pub fn controller(&self) -> &C {
        &self.controller
    }

    /// Synchronizes bounds, visibility, parking, and focus policy.
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

    /// Runs a caller-provided event pump, synchronizes the area, drains webview
    /// events, and applies built-in focus policy for those events.
    pub fn tick_with<F>(
        &mut self,
        state: WebViewAreaState,
        pump_events: F,
    ) -> Result<Vec<WebViewEvent>>
    where
        F: FnOnce(),
    {
        pump_events();
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

    /// Allows keyboard focus and requests focus for the webview when the area is
    /// effectively visible and no shell input owns focus.
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

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;
    use crate::{CompositionTier, ScriptRequestId, WebViewCapabilities};

    #[derive(Default)]
    struct RecordingController {
        operations: RefCell<Vec<String>>,
        events: RefCell<Vec<WebViewEvent>>,
        next_script_request_id: Cell<u64>,
    }

    impl RecordingController {
        fn operations(&self) -> Vec<String> {
            self.operations.borrow().clone()
        }

        fn queue_event(&self, event: WebViewEvent) {
            self.events.borrow_mut().push(event);
        }
    }

    impl WebViewControllerLike for RecordingController {
        fn capabilities() -> WebViewCapabilities {
            WebViewCapabilities {
                backend_name: "recording",
                engine_name: "test",
                composition_tier: CompositionTier::SlintOwnedTexture,
                supports_transparency: true,
                supports_clipping: true,
                supports_overlays_above_webview: true,
                supports_script_eval: true,
                supports_host_messaging: true,
                supports_custom_user_agent: true,
                supports_download_interception: false,
                supports_permission_interception: false,
                requires_external_runtime: false,
            }
        }

        fn drain_events(&self) -> Vec<WebViewEvent> {
            self.events.borrow_mut().drain(..).collect()
        }

        fn set_bounds(&self, bounds: WebViewBounds) -> Result<()> {
            self.operations
                .borrow_mut()
                .push(format!("bounds:{}x{}", bounds.width, bounds.height));
            Ok(())
        }

        fn set_visible(&self, visible: bool) -> Result<()> {
            self.operations
                .borrow_mut()
                .push(format!("visible:{visible}"));
            Ok(())
        }

        fn load_source(&self, source: WebViewSource) -> Result<()> {
            self.operations
                .borrow_mut()
                .push(format!("source:{source:?}"));
            Ok(())
        }

        fn load_html(&self, html: &str) -> Result<()> {
            self.operations.borrow_mut().push(format!("html:{html}"));
            Ok(())
        }

        fn load_url(&self, url: &str) -> Result<()> {
            self.operations.borrow_mut().push(format!("url:{url}"));
            Ok(())
        }

        fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId> {
            let request_id = ScriptRequestId(self.next_script_request_id.get());
            self.next_script_request_id
                .set(request_id.0.saturating_add(1).max(1));
            self.operations
                .borrow_mut()
                .push(format!("script:{}:{script}", request_id.0));
            Ok(request_id)
        }

        fn focus(&self) -> Result<()> {
            self.operations.borrow_mut().push("focus".to_owned());
            Ok(())
        }

        fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
            self.operations
                .borrow_mut()
                .push(format!("keyboard:{enabled}"));
            Ok(())
        }

        fn focus_parent(&self) -> Result<()> {
            self.operations.borrow_mut().push("focus-parent".to_owned());
            Ok(())
        }
    }

    #[test]
    fn default_policy_parks_when_overlay_is_active() {
        let state = WebViewAreaState::new(WebViewBounds::new(10.0, 20.0, 300.0, 200.0))
            .with_overlay_active(true);
        let placement = WebViewAreaPolicy::default().resolve(state);

        assert!(!placement.effective_visible);
        assert!(placement.native_visible);
        assert!(placement.parked);
        assert!(placement.blocked_by_overlay);
        assert_eq!(placement.bounds, DEFAULT_PARK_BOUNDS);
    }

    #[test]
    fn hide_strategy_uses_native_visibility_without_parking() {
        let policy = WebViewAreaPolicy::default().with_hidden_strategy(HiddenWebViewStrategy::Hide);
        let state = WebViewAreaState::new(WebViewBounds::new(10.0, 20.0, 300.0, 200.0))
            .with_requested_visible(false);
        let placement = policy.resolve(state);

        assert!(!placement.effective_visible);
        assert!(!placement.native_visible);
        assert!(!placement.parked);
        assert_eq!(placement.bounds, state.bounds);
    }

    #[test]
    fn invalid_bounds_are_never_applied_to_backend() {
        let state = WebViewAreaState::new(WebViewBounds::new(1.0, 2.0, 0.0, 4.0));
        let placement = WebViewAreaPolicy::default().resolve(state);

        assert!(!placement.effective_visible);
        assert!(placement.blocked_by_invalid_bounds);
        assert_eq!(placement.bounds, DEFAULT_PARK_BOUNDS);
    }

    #[test]
    fn invalid_custom_park_bounds_fall_back_to_default() {
        let policy =
            WebViewAreaPolicy::default().with_park_bounds(WebViewBounds::new(-1.0, -1.0, 0.0, 0.0));
        let state = WebViewAreaState::default().with_requested_visible(false);
        let placement = policy.resolve(state);

        assert_eq!(placement.bounds, DEFAULT_PARK_BOUNDS);
    }

    #[test]
    fn area_controller_applies_initial_placement() {
        let state = WebViewAreaState::new(WebViewBounds::new(10.0, 20.0, 300.0, 200.0));
        let area = BackendWebViewAreaController::from_controller(
            RecordingController::default(),
            state,
            WebViewAreaPolicy::default(),
        )
        .unwrap();

        assert_eq!(
            area.controller().operations(),
            vec!["bounds:300x200", "visible:true", "keyboard:false"]
        );
        assert!(area.status().placement.effective_visible);
        assert!(!area.status().keyboard_focus_enabled);
    }

    #[test]
    fn area_controller_releases_focus_when_hidden() {
        let state = WebViewAreaState::new(WebViewBounds::new(10.0, 20.0, 300.0, 200.0));
        let mut area = BackendWebViewAreaController::from_controller_with_keyboard_focus(
            RecordingController::default(),
            state,
            WebViewAreaPolicy::default(),
            true,
        )
        .unwrap();

        area.sync(state.with_overlay_active(true)).unwrap();

        assert_eq!(
            area.controller().operations(),
            vec![
                "bounds:300x200",
                "visible:true",
                "keyboard:true",
                "bounds:1x1",
                "keyboard:false",
                "focus-parent",
            ]
        );
        assert!(!area.status().placement.effective_visible);
        assert!(!area.status().keyboard_focus_enabled);
    }

    #[test]
    fn area_controller_applies_focus_requested_event_policy() {
        let state = WebViewAreaState::new(WebViewBounds::new(10.0, 20.0, 300.0, 200.0));
        let mut area = BackendWebViewAreaController::from_controller(
            RecordingController::default(),
            state,
            WebViewAreaPolicy::default(),
        )
        .unwrap();
        area.controller().queue_event(WebViewEvent::FocusRequested);

        assert_eq!(
            area.drain_events().unwrap(),
            vec![WebViewEvent::FocusRequested]
        );
        assert_eq!(
            area.controller().operations(),
            vec![
                "bounds:300x200",
                "visible:true",
                "keyboard:false",
                "keyboard:true",
                "focus",
            ]
        );
        assert!(area.status().keyboard_focus_enabled);
    }

    #[test]
    fn area_controller_uses_caller_event_pump() {
        let pumped = Cell::new(false);
        let state = WebViewAreaState::new(WebViewBounds::new(10.0, 20.0, 300.0, 200.0));
        let mut area = BackendWebViewAreaController::from_controller(
            RecordingController::default(),
            state,
            WebViewAreaPolicy::default(),
        )
        .unwrap();

        area.tick_with(state, || pumped.set(true)).unwrap();

        assert!(pumped.get());
    }
}
