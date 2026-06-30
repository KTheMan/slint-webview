use std::cell::Cell;
use std::sync::mpsc::Receiver;

use crate::{
    Result, ScriptRequestId, WebViewBounds, WebViewCapabilities, WebViewError, WebViewEvent,
    WebViewSource,
};

/// Browser-engine operations required by the shared controller.
///
/// Backend crates implement this trait for their concrete native, Servo, or CEF
/// webview instance. The shared controller owns event draining, script request
/// IDs, source dispatch, and bounds validation so those behaviors stay uniform
/// across backends.
pub trait WebViewBackend {
    /// Returns static capabilities for this backend.
    fn capabilities() -> WebViewCapabilities
    where
        Self: Sized;

    /// Updates backend bounds in Slint logical window coordinates.
    fn set_bounds(&self, bounds: WebViewBounds) -> Result<()>;

    /// Shows or hides the backend surface.
    fn set_visible(&self, visible: bool) -> Result<()>;

    /// Loads an HTML string.
    fn load_html(&self, html: &str) -> Result<()>;

    /// Loads a URL.
    fn load_url(&self, url: &str) -> Result<()>;

    /// Evaluates JavaScript and reports the result through the shared event
    /// stream using the provided request ID.
    fn evaluate_script(&self, script: &str, request_id: ScriptRequestId) -> Result<()>;

    /// Requests keyboard focus for the backend.
    fn focus(&self) -> Result<()>;

    /// Enables or disables whether the backend may take keyboard focus.
    fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()>;

    /// Returns focus to the host shell where supported.
    fn focus_parent(&self) -> Result<()>;
}

/// Shared controller for a concrete webview backend.
///
/// This controller intentionally has no Slint or platform-window dependency.
/// Backend crates are responsible for creation/attachment, then hand the
/// backend instance and event receiver to this type.
pub struct BackendWebViewController<B> {
    backend: B,
    events: Receiver<WebViewEvent>,
    next_script_request_id: Cell<u64>,
}

impl<B> BackendWebViewController<B>
where
    B: WebViewBackend,
{
    /// Creates a controller from a backend instance and its event receiver.
    pub fn new(backend: B, events: Receiver<WebViewEvent>) -> Self {
        Self {
            backend,
            events,
            next_script_request_id: Cell::new(1),
        }
    }

    /// Returns the capabilities of the selected backend.
    pub fn capabilities() -> WebViewCapabilities {
        B::capabilities()
    }

    /// Returns the underlying backend instance.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Attempts to receive one pending event.
    pub fn try_recv_event(&self) -> Option<WebViewEvent> {
        self.events.try_recv().ok()
    }

    /// Drains all events currently queued for this controller.
    pub fn drain_events(&self) -> Vec<WebViewEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.try_recv_event() {
            events.push(event);
        }
        events
    }

    /// Updates webview bounds in Slint logical window coordinates.
    pub fn set_bounds(&self, bounds: WebViewBounds) -> Result<()> {
        validate_bounds(bounds)?;
        self.backend.set_bounds(bounds)
    }

    /// Shows or hides the backend surface.
    pub fn set_visible(&self, visible: bool) -> Result<()> {
        self.backend.set_visible(visible)
    }

    /// Loads a source into the webview.
    pub fn load_source(&self, source: WebViewSource) -> Result<()> {
        match source {
            WebViewSource::Blank => self.load_html(""),
            WebViewSource::Url(url) => self.load_url(&url),
            WebViewSource::Html(html) => self.load_html(&html),
        }
    }

    /// Loads an HTML string into the webview.
    pub fn load_html(&self, html: &str) -> Result<()> {
        self.backend.load_html(html)
    }

    /// Loads a URL into the webview.
    pub fn load_url(&self, url: &str) -> Result<()> {
        self.backend.load_url(url)
    }

    /// Evaluates JavaScript and returns the request ID that will appear on the
    /// matching [`WebViewEvent::ScriptResult`] event.
    pub fn evaluate_script(&self, script: &str) -> Result<ScriptRequestId> {
        let request_id = self.allocate_script_request_id();
        self.backend.evaluate_script(script, request_id)?;
        Ok(request_id)
    }

    /// Requests keyboard focus for the backend.
    pub fn focus(&self) -> Result<()> {
        self.backend.focus()
    }

    /// Enables or disables whether the backend may take keyboard focus.
    pub fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
        self.backend.set_keyboard_focus_enabled(enabled)
    }

    /// Returns focus to the host shell where supported.
    pub fn focus_parent(&self) -> Result<()> {
        self.backend.focus_parent()
    }

    fn allocate_script_request_id(&self) -> ScriptRequestId {
        let id = self.next_script_request_id.get();
        self.next_script_request_id.set(id.saturating_add(1).max(1));
        ScriptRequestId(id)
    }
}

/// Validates webview bounds before they are handed to backend implementations.
pub fn validate_bounds(bounds: WebViewBounds) -> Result<()> {
    if bounds.is_valid() {
        Ok(())
    } else {
        Err(WebViewError::InvalidBounds(bounds))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::mpsc;

    use super::*;
    use crate::{CompositionTier, WebViewCapabilities};

    #[derive(Default)]
    struct RecordingBackend {
        operations: RefCell<Vec<String>>,
    }

    impl RecordingBackend {
        fn operations(&self) -> Vec<String> {
            self.operations.borrow().clone()
        }
    }

    impl WebViewBackend for RecordingBackend {
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

        fn load_html(&self, html: &str) -> Result<()> {
            self.operations.borrow_mut().push(format!("html:{html}"));
            Ok(())
        }

        fn load_url(&self, url: &str) -> Result<()> {
            self.operations.borrow_mut().push(format!("url:{url}"));
            Ok(())
        }

        fn evaluate_script(&self, script: &str, request_id: ScriptRequestId) -> Result<()> {
            self.operations
                .borrow_mut()
                .push(format!("script:{}:{script}", request_id.0));
            Ok(())
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
    fn source_loading_is_backend_agnostic() {
        let (_sender, receiver) = mpsc::channel();
        let controller = BackendWebViewController::new(RecordingBackend::default(), receiver);

        controller.load_source(WebViewSource::Blank).unwrap();
        controller
            .load_source(WebViewSource::Html("<p>hello</p>".to_owned()))
            .unwrap();
        controller
            .load_source(WebViewSource::Url("https://example.com".to_owned()))
            .unwrap();

        assert_eq!(
            controller.backend().operations(),
            vec!["html:", "html:<p>hello</p>", "url:https://example.com"]
        );
    }

    #[test]
    fn script_request_ids_are_allocated_by_core() {
        let (_sender, receiver) = mpsc::channel();
        let controller = BackendWebViewController::new(RecordingBackend::default(), receiver);

        assert_eq!(
            controller.evaluate_script("1 + 1").unwrap(),
            ScriptRequestId(1)
        );
        assert_eq!(
            controller.evaluate_script("2 + 2").unwrap(),
            ScriptRequestId(2)
        );
        assert_eq!(
            controller.backend().operations(),
            vec!["script:1:1 + 1", "script:2:2 + 2"]
        );
    }

    #[test]
    fn events_are_drained_by_core() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(WebViewEvent::TitleChanged {
                title: "ready".to_owned(),
            })
            .unwrap();
        let controller = BackendWebViewController::new(RecordingBackend::default(), receiver);

        assert_eq!(
            controller.drain_events(),
            vec![WebViewEvent::TitleChanged {
                title: "ready".to_owned()
            }]
        );
        assert!(controller.drain_events().is_empty());
    }

    #[test]
    fn invalid_bounds_are_rejected_before_backend_call() {
        let (_sender, receiver) = mpsc::channel();
        let controller = BackendWebViewController::new(RecordingBackend::default(), receiver);

        assert!(matches!(
            controller.set_bounds(WebViewBounds::new(0.0, 0.0, 0.0, 10.0)),
            Err(WebViewError::InvalidBounds(_))
        ));
        assert!(controller.backend().operations().is_empty());
    }
}
