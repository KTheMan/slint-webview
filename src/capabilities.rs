use serde::{Deserialize, Serialize};

/// Composition strategy used by a backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositionTier {
    /// No native webview backend is available.
    Unsupported,
    /// Native child view or child window hosted inside the parent window.
    NativeChildView,
    /// Platform compositor integration, where available.
    PlatformVisualHosting,
    /// Web content rendered into a Slint-owned texture.
    SlintOwnedTexture,
}

/// Runtime and backend capabilities visible to application code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebViewCapabilities {
    /// Backend implementation name.
    pub backend_name: &'static str,
    /// Platform engine name.
    pub engine_name: &'static str,
    /// Composition strategy.
    pub composition_tier: CompositionTier,
    /// Whether transparent webviews are supported.
    pub supports_transparency: bool,
    /// Whether the backend supports normal Slint clipping semantics.
    pub supports_clipping: bool,
    /// Whether Slint overlays can reliably appear above the webview.
    pub supports_overlays_above_webview: bool,
    /// Whether JavaScript evaluation is supported.
    pub supports_script_eval: bool,
    /// Whether web-to-host messaging is supported.
    pub supports_host_messaging: bool,
    /// Whether custom user agents are supported.
    pub supports_custom_user_agent: bool,
    /// Whether downloads can be intercepted.
    pub supports_download_interception: bool,
    /// Whether permission prompts can be intercepted.
    pub supports_permission_interception: bool,
    /// Whether the platform may require an external runtime package.
    pub requires_external_runtime: bool,
}

impl WebViewCapabilities {
    /// Capabilities for the Wry native-child backend.
    pub const fn wry_native() -> Self {
        Self {
            backend_name: "wry",
            engine_name: platform_engine_name(),
            composition_tier: CompositionTier::NativeChildView,
            supports_transparency: false,
            supports_clipping: false,
            supports_overlays_above_webview: false,
            supports_script_eval: true,
            supports_host_messaging: true,
            supports_custom_user_agent: true,
            supports_download_interception: true,
            supports_permission_interception: false,
            requires_external_runtime: platform_requires_external_runtime(),
        }
    }

    /// Capabilities reported when no backend is compiled.
    pub const fn unsupported() -> Self {
        Self {
            backend_name: "none",
            engine_name: "none",
            composition_tier: CompositionTier::Unsupported,
            supports_transparency: false,
            supports_clipping: false,
            supports_overlays_above_webview: false,
            supports_script_eval: false,
            supports_host_messaging: false,
            supports_custom_user_agent: false,
            supports_download_interception: false,
            supports_permission_interception: false,
            requires_external_runtime: false,
        }
    }
}

const fn platform_engine_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "WebView2"
    } else if cfg!(target_os = "macos") {
        "WKWebView"
    } else if cfg!(target_os = "linux") {
        "WebKitGTK"
    } else {
        "native-webview"
    }
}

const fn platform_requires_external_runtime() -> bool {
    cfg!(target_os = "windows") || cfg!(target_os = "linux")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wry_native_is_child_view_tier() {
        let caps = WebViewCapabilities::wry_native();

        assert_eq!(caps.backend_name, "wry");
        assert_eq!(caps.composition_tier, CompositionTier::NativeChildView);
        assert!(caps.supports_script_eval);
    }
}
