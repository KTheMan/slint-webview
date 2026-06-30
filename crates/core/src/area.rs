use serde::{Deserialize, Serialize};

use crate::WebViewBounds;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
