#[cfg(feature = "backend-wry")]
mod wry_backend;

#[cfg(feature = "backend-wry")]
pub use wry_backend::{NativeWebView, initialize_platform, pump_platform_events};

#[cfg(not(feature = "backend-wry"))]
pub struct NativeWebView;

#[cfg(not(feature = "backend-wry"))]
impl NativeWebView {
    pub fn attach<W>(
        _window: &W,
        _options: crate::WebViewOptions,
        _event_sender: std::sync::mpsc::Sender<crate::WebViewEvent>,
    ) -> crate::Result<Self>
    where
        W: raw_window_handle::HasWindowHandle,
    {
        Err(crate::WebViewError::BackendDisabled)
    }

    pub fn capabilities() -> crate::WebViewCapabilities {
        crate::WebViewCapabilities::unsupported()
    }

    pub fn set_bounds(&self, _bounds: crate::WebViewBounds) -> crate::Result<()> {
        Err(crate::WebViewError::BackendDisabled)
    }

    pub fn set_visible(&self, _visible: bool) -> crate::Result<()> {
        Err(crate::WebViewError::BackendDisabled)
    }

    pub fn load_html(&self, _html: &str) -> crate::Result<()> {
        Err(crate::WebViewError::BackendDisabled)
    }

    pub fn load_url(&self, _url: &str) -> crate::Result<()> {
        Err(crate::WebViewError::BackendDisabled)
    }

    pub fn evaluate_script(
        &self,
        _script: &str,
        _request_id: crate::ScriptRequestId,
    ) -> crate::Result<()> {
        Err(crate::WebViewError::BackendDisabled)
    }

    pub fn focus(&self) -> crate::Result<()> {
        Err(crate::WebViewError::BackendDisabled)
    }

    pub fn set_keyboard_focus_enabled(&self, _enabled: bool) -> crate::Result<()> {
        Err(crate::WebViewError::BackendDisabled)
    }

    pub fn focus_parent(&self) -> crate::Result<()> {
        Err(crate::WebViewError::BackendDisabled)
    }
}

#[cfg(not(feature = "backend-wry"))]
pub fn initialize_platform() -> crate::Result<()> {
    Err(crate::WebViewError::BackendDisabled)
}

#[cfg(not(feature = "backend-wry"))]
pub fn pump_platform_events() {}
