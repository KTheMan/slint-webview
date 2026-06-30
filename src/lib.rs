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

mod area;
mod controller;

pub use area::WebViewAreaController;
pub use controller::{WebViewController, initialize_platform, pump_platform_events};
#[cfg(feature = "testing")]
pub use slint_webview_core::fixture;
pub use slint_webview_core::{
    BackendRenderedWebViewController, BackendWebViewAreaController, BackendWebViewController,
    CompositionTier, DEFAULT_PARK_BOUNDS, HiddenWebViewStrategy, NavigationDecision,
    NavigationPolicy, RenderedWebViewBackend, RenderedWebViewCapabilities,
    RenderedWebViewDirtyRect, RenderedWebViewFrame, RenderedWebViewFrameId,
    RenderedWebViewFramePayload, RenderedWebViewFrameTransport, RenderedWebViewInputEvent,
    RenderedWebViewInputState, RenderedWebViewModifiers, RenderedWebViewPixelFormat,
    RenderedWebViewPointerButton, RenderedWebViewSize, RenderedWebViewTextureApi, Result,
    ScriptRequestId, WebViewAreaPlacement, WebViewAreaPolicy, WebViewAreaState, WebViewAreaStatus,
    WebViewBackend, WebViewBounds, WebViewCapabilities, WebViewControllerLike, WebViewError,
    WebViewEvent, WebViewOptions, WebViewSource, validate_bounds, validate_rendered_frame,
    validate_rendered_size,
};
