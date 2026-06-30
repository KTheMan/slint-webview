//! Shared API and policy types for Slint webview backends.
//!
//! `slint-webview-core` is intentionally backend-free. Native, Servo, and CEF
//! crates should depend on these types so application-facing behavior remains
//! uniform across different rendering engines.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod api;
mod area;
mod backend;
mod capabilities;
mod error;
#[cfg(any(test, feature = "testing"))]
pub mod fixture;

pub use api::{
    NavigationDecision, NavigationPolicy, ScriptRequestId, WebViewBounds, WebViewEvent,
    WebViewOptions, WebViewSource,
};
pub use area::{
    BackendWebViewAreaController, DEFAULT_PARK_BOUNDS, HiddenWebViewStrategy, WebViewAreaPlacement,
    WebViewAreaPolicy, WebViewAreaState, WebViewAreaStatus, WebViewControllerLike,
};
pub use backend::{BackendWebViewController, WebViewBackend, validate_bounds};
pub use capabilities::{CompositionTier, WebViewCapabilities};
pub use error::{Result, WebViewError};
