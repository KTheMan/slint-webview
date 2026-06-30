use serde::{Deserialize, Serialize};

/// Content to load into a webview.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum WebViewSource {
    /// Start with an empty document.
    #[default]
    Blank,
    /// Load a URL through the platform webview.
    Url(String),
    /// Load an HTML string directly.
    Html(String),
}

/// A rectangle in Slint logical window coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WebViewBounds {
    /// Left coordinate in logical pixels.
    pub x: f64,
    /// Top coordinate in logical pixels.
    pub y: f64,
    /// Width in logical pixels.
    pub width: f64,
    /// Height in logical pixels.
    pub height: f64,
}

impl WebViewBounds {
    /// Creates a bounds rectangle in logical pixels.
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns a copy with width and height clamped to the provided minimums.
    pub fn clamped(self, min_width: f64, min_height: f64) -> Self {
        Self {
            width: self.width.max(min_width),
            height: self.height.max(min_height),
            ..self
        }
    }

    /// Returns true when all coordinates are finite and the size is positive.
    pub fn is_valid(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

impl Default for WebViewBounds {
    fn default() -> Self {
        Self::new(300.0, 64.0, 640.0, 480.0)
    }
}

/// Navigation policy applied before a navigation is allowed to continue.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationPolicy {
    /// Allow all navigation.
    #[default]
    AllowAll,
    /// Block all navigation.
    BlockAll,
    /// Block URLs whose scheme matches one of the listed schemes.
    BlockSchemes(Vec<String>),
    /// Allow only URLs whose scheme matches one of the listed schemes.
    AllowSchemes(Vec<String>),
}

impl NavigationPolicy {
    /// Applies this policy to a URL string.
    pub fn decide(&self, url: &str) -> NavigationDecision {
        match self {
            Self::AllowAll => NavigationDecision::Allow,
            Self::BlockAll => NavigationDecision::Block,
            Self::BlockSchemes(schemes) => {
                if schemes.iter().any(|scheme| url_has_scheme(url, scheme)) {
                    NavigationDecision::Block
                } else {
                    NavigationDecision::Allow
                }
            }
            Self::AllowSchemes(schemes) => {
                if schemes.iter().any(|scheme| url_has_scheme(url, scheme)) {
                    NavigationDecision::Allow
                } else {
                    NavigationDecision::Block
                }
            }
        }
    }
}

/// Options used when creating a webview.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebViewOptions {
    /// Initial content.
    pub source: WebViewSource,
    /// Initial rectangle in Slint logical window coordinates.
    pub bounds: WebViewBounds,
    /// Enables JavaScript execution.
    pub javascript_enabled: bool,
    /// Enables backend developer tools where the platform supports them.
    pub devtools: bool,
    /// Initial visibility.
    pub visible: bool,
    /// Whether the native webview should take keyboard focus when attached.
    pub focused: bool,
    /// Allows web content to create new windows.
    pub allow_popups: bool,
    /// Allows web content to start file downloads.
    pub downloads_enabled: bool,
    /// Allows the webview to access the platform clipboard.
    pub clipboard_enabled: bool,
    /// Runs the webview in an ephemeral/private profile where supported.
    pub incognito: bool,
    /// Optional user agent override.
    pub user_agent: Option<String>,
    /// JavaScript snippets injected before page scripts run.
    pub initialization_scripts: Vec<String>,
    /// Maximum IPC message body length accepted from web content.
    pub ipc_message_limit: usize,
    /// Navigation policy for top-level navigation requests.
    pub navigation_policy: NavigationPolicy,
}

impl Default for WebViewOptions {
    fn default() -> Self {
        Self {
            source: WebViewSource::default(),
            bounds: WebViewBounds::default(),
            javascript_enabled: false,
            devtools: false,
            visible: true,
            focused: false,
            allow_popups: false,
            downloads_enabled: false,
            clipboard_enabled: false,
            incognito: false,
            user_agent: None,
            initialization_scripts: Vec::new(),
            ipc_message_limit: 1024 * 1024,
            navigation_policy: NavigationPolicy::default(),
        }
    }
}

impl WebViewOptions {
    /// Returns options with a different initial source.
    pub fn with_source(mut self, source: WebViewSource) -> Self {
        self.source = source;
        self
    }

    /// Returns options with different initial bounds.
    pub fn with_bounds(mut self, bounds: WebViewBounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// Returns options with JavaScript enabled or disabled.
    pub fn with_javascript_enabled(mut self, enabled: bool) -> Self {
        self.javascript_enabled = enabled;
        self
    }

    /// Returns options with developer tools enabled or disabled.
    pub fn with_devtools(mut self, enabled: bool) -> Self {
        self.devtools = enabled;
        self
    }

    /// Returns options with initial visibility set.
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Returns options with initial webview focus enabled or disabled.
    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Returns options with popup window creation enabled or disabled.
    pub fn with_popups_enabled(mut self, enabled: bool) -> Self {
        self.allow_popups = enabled;
        self
    }

    /// Returns options with file downloads enabled or disabled.
    pub fn with_downloads_enabled(mut self, enabled: bool) -> Self {
        self.downloads_enabled = enabled;
        self
    }

    /// Returns options with clipboard access enabled or disabled.
    pub fn with_clipboard_enabled(mut self, enabled: bool) -> Self {
        self.clipboard_enabled = enabled;
        self
    }

    /// Returns options with incognito/private profile mode enabled or disabled.
    pub fn with_incognito(mut self, enabled: bool) -> Self {
        self.incognito = enabled;
        self
    }

    /// Returns options with a user agent override.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Returns options with an additional initialization script.
    pub fn with_initialization_script(mut self, script: impl Into<String>) -> Self {
        self.initialization_scripts.push(script.into());
        self
    }

    /// Returns options with a different IPC message byte limit.
    pub fn with_ipc_message_limit(mut self, limit: usize) -> Self {
        self.ipc_message_limit = limit;
        self
    }

    /// Returns options with a different navigation policy.
    pub fn with_navigation_policy(mut self, policy: NavigationPolicy) -> Self {
        self.navigation_policy = policy;
        self
    }
}

/// A decision produced by a navigation policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationDecision {
    /// Let the webview navigate normally.
    Allow,
    /// Cancel the navigation.
    Block,
    /// Cancel the navigation and let the application open it externally.
    OpenExternal,
}

/// Identifier for an asynchronous script evaluation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptRequestId(pub u64);

/// Event emitted by a webview controller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebViewEvent {
    /// Page load started.
    NavigationStarted {
        /// URL being loaded.
        url: String,
    },
    /// Page load finished.
    NavigationFinished {
        /// URL that finished loading.
        url: String,
    },
    /// Navigation was requested and a policy decision was made.
    NavigationRequested {
        /// Requested URL.
        url: String,
        /// Policy decision.
        decision: NavigationDecision,
    },
    /// Web content sent an IPC message to the host.
    IpcMessage {
        /// Current document URI reported by the backend.
        uri: String,
        /// Message body.
        body: String,
        /// True when the message body exceeded the configured limit.
        truncated: bool,
    },
    /// Document title changed.
    TitleChanged {
        /// New title.
        title: String,
    },
    /// The native webview received an input action that should be allowed to
    /// claim keyboard focus.
    FocusRequested,
    /// Native keyboard focus entered or left the webview.
    FocusChanged {
        /// True when the webview currently owns native keyboard focus.
        focused: bool,
    },
    /// Script evaluation finished.
    ScriptResult {
        /// Matching request ID.
        request_id: ScriptRequestId,
        /// Serialized script result.
        value: String,
    },
    /// New-window request was handled by policy.
    NewWindowRequested {
        /// Requested URL.
        url: String,
        /// True if the platform backend was allowed to create a window.
        allowed: bool,
    },
    /// File download was requested.
    DownloadRequested {
        /// Download URL.
        url: String,
        /// Backend-suggested destination path, when known.
        suggested_path: Option<String>,
        /// True if the download was allowed to continue.
        allowed: bool,
    },
    /// File download completed.
    DownloadFinished {
        /// Download URL.
        url: String,
        /// Final path reported by the backend, when known.
        path: Option<String>,
        /// True when the backend reported success.
        success: bool,
    },
}

fn url_has_scheme(url: &str, scheme: &str) -> bool {
    url.split_once(':')
        .is_some_and(|(actual, _)| actual.eq_ignore_ascii_case(scheme))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_clamp_preserves_position() {
        let bounds = WebViewBounds::new(10.0, 20.0, 1.0, 2.0).clamped(300.0, 200.0);

        assert_eq!(bounds.x, 10.0);
        assert_eq!(bounds.y, 20.0);
        assert_eq!(bounds.width, 300.0);
        assert_eq!(bounds.height, 200.0);
    }

    #[test]
    fn default_options_are_conservative() {
        let options = WebViewOptions::default();

        assert!(!options.javascript_enabled);
        assert!(!options.devtools);
        assert!(!options.focused);
        assert!(!options.clipboard_enabled);
        assert!(!options.downloads_enabled);
        assert!(!options.incognito);
        assert!(options.user_agent.is_none());
        assert!(options.initialization_scripts.is_empty());
        assert!(matches!(options.source, WebViewSource::Blank));
    }

    #[test]
    fn option_builders_update_expected_fields() {
        let options = WebViewOptions::default()
            .with_source(WebViewSource::Url("https://example.com".to_owned()))
            .with_javascript_enabled(true)
            .with_focused(true)
            .with_downloads_enabled(true)
            .with_user_agent("slint-webview-test")
            .with_initialization_script("window.__ready = true");

        assert!(matches!(options.source, WebViewSource::Url(_)));
        assert!(options.javascript_enabled);
        assert!(options.focused);
        assert!(options.downloads_enabled);
        assert_eq!(options.user_agent.as_deref(), Some("slint-webview-test"));
        assert_eq!(options.initialization_scripts.len(), 1);
    }

    #[test]
    fn navigation_policy_blocks_scheme_case_insensitively() {
        let policy = NavigationPolicy::BlockSchemes(vec!["slint-blocked".to_owned()]);

        assert_eq!(
            policy.decide("SLINT-BLOCKED:navigation"),
            NavigationDecision::Block
        );
        assert_eq!(
            policy.decide("https://example.com"),
            NavigationDecision::Allow
        );
    }
}
