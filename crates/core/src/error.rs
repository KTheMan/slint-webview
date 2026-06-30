use crate::WebViewBounds;

/// Error returned by webview operations.
#[derive(Debug, thiserror::Error)]
pub enum WebViewError {
    /// The selected backend was not compiled in.
    #[error("native webview backend is disabled")]
    BackendDisabled,
    /// A native window handle is not available yet.
    #[error("window handle is not available yet: {0}")]
    WindowHandle(String),
    /// Bounds are invalid.
    #[error("webview bounds are invalid: {0:?}")]
    InvalidBounds(WebViewBounds),
    /// Navigation was blocked by policy.
    #[error("navigation was blocked by policy: {0}")]
    NavigationBlocked(String),
    /// A native backend operation failed.
    #[error("native webview operation failed: {0}")]
    Native(String),
    /// Platform setup failed.
    #[error("platform setup failed: {0}")]
    Platform(String),
}

/// Result type used by this crate.
pub type Result<T> = std::result::Result<T, WebViewError>;
