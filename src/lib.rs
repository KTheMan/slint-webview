//! Native webview controller for Slint applications.
//!
//! `slint-webview` embeds the platform webview as a native child surface inside
//! a Slint window. The default backend is Wry, which uses WebView2 on Windows,
//! WebKitGTK on Linux, and the platform webview on other supported targets.
//!
//! The webview is a native child view, not a normal Slint scene-graph item.
//! That keeps application size small, but it also means clipping, transforms,
//! opacity, and overlays need explicit policy handling.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod api;
mod area;
mod capabilities;
mod controller;
mod error;
#[cfg(any(test, feature = "testing"))]
pub mod fixture;
mod platform;

pub use api::{
    NavigationDecision, NavigationPolicy, ScriptRequestId, WebViewBounds, WebViewEvent,
    WebViewOptions, WebViewSource,
};
pub use area::{
    DEFAULT_PARK_BOUNDS, HiddenWebViewStrategy, WebViewAreaController, WebViewAreaPlacement,
    WebViewAreaPolicy, WebViewAreaState, WebViewAreaStatus,
};
pub use capabilities::{CompositionTier, WebViewCapabilities};
pub use controller::{WebViewController, initialize_platform, pump_platform_events};
pub use error::{Result, WebViewError};
