//! Deterministic rendered backend for Slint webview tests and examples.
//!
//! This crate is intentionally not a browser engine. It implements the shared
//! backend contracts with predictable events and CPU frames so Slint-side
//! composition, input routing, and controller behavior can be tested without a
//! native webview, CEF, or Servo runtime.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};

pub use slint_webview_core::{
    BackendRenderedWebViewController, BackendWebViewController, CompositionTier,
    RenderedWebViewBackend, RenderedWebViewCapabilities, RenderedWebViewFrame,
    RenderedWebViewFrameId, RenderedWebViewFramePayload, RenderedWebViewFrameTransport,
    RenderedWebViewInputEvent, RenderedWebViewInputState, RenderedWebViewPixelFormat,
    RenderedWebViewSize, Result, ScriptRequestId, WebViewBackend, WebViewBounds,
    WebViewCapabilities, WebViewError, WebViewEvent, validate_rendered_frame,
};

const MAX_MOCK_FRAME_BYTES: usize = 64 * 1024 * 1024;

/// A rendered backend that records operations and produces deterministic frames.
pub struct MockRenderedWebView {
    event_sender: Sender<WebViewEvent>,
    bounds: RefCell<Option<WebViewBounds>>,
    visible: Cell<bool>,
    keyboard_focus_enabled: Cell<bool>,
    html: RefCell<Option<String>>,
    url: RefCell<Option<String>>,
    scripts: RefCell<Vec<(ScriptRequestId, String)>>,
    sizes: RefCell<Vec<RenderedWebViewSize>>,
    inputs: RefCell<Vec<RenderedWebViewInputEvent>>,
    frames: RefCell<VecDeque<RenderedWebViewFrame>>,
    next_frame_id: Cell<u64>,
}

impl MockRenderedWebView {
    /// Creates a mock backend and a matching event receiver.
    pub fn new() -> (Self, Receiver<WebViewEvent>) {
        let (event_sender, receiver) = mpsc::channel();
        (
            Self {
                event_sender,
                bounds: RefCell::new(None),
                visible: Cell::new(true),
                keyboard_focus_enabled: Cell::new(false),
                html: RefCell::new(None),
                url: RefCell::new(None),
                scripts: RefCell::new(Vec::new()),
                sizes: RefCell::new(Vec::new()),
                inputs: RefCell::new(Vec::new()),
                frames: RefCell::new(VecDeque::new()),
                next_frame_id: Cell::new(1),
            },
            receiver,
        )
    }

    /// Creates a mock rendered controller.
    pub fn controller() -> MockRenderedWebViewController {
        let (backend, receiver) = Self::new();
        BackendRenderedWebViewController::new(backend, receiver)
    }

    /// Queues a validated frame for the next controller drain.
    pub fn queue_frame(&self, frame: RenderedWebViewFrame) -> Result<()> {
        validate_rendered_frame(&frame)?;
        self.frames.borrow_mut().push_back(frame);
        Ok(())
    }

    /// Returns the last logical bounds sent to the backend.
    pub fn last_bounds(&self) -> Option<WebViewBounds> {
        *self.bounds.borrow()
    }

    /// Returns whether the mock surface is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    /// Returns whether the mock backend may claim keyboard focus.
    pub fn keyboard_focus_enabled(&self) -> bool {
        self.keyboard_focus_enabled.get()
    }

    /// Returns the last HTML string loaded by the controller.
    pub fn last_html(&self) -> Option<String> {
        self.html.borrow().clone()
    }

    /// Returns the last URL loaded by the controller.
    pub fn last_url(&self) -> Option<String> {
        self.url.borrow().clone()
    }

    /// Returns recorded script evaluations.
    pub fn scripts(&self) -> Vec<(ScriptRequestId, String)> {
        self.scripts.borrow().clone()
    }

    /// Returns recorded rendered-surface sizes.
    pub fn sizes(&self) -> Vec<RenderedWebViewSize> {
        self.sizes.borrow().clone()
    }

    /// Returns recorded rendered input events.
    pub fn inputs(&self) -> Vec<RenderedWebViewInputEvent> {
        self.inputs.borrow().clone()
    }

    /// Returns the number of frames currently waiting to be drained.
    pub fn queued_frame_count(&self) -> usize {
        self.frames.borrow().len()
    }

    fn emit(&self, event: WebViewEvent) {
        let _ = self.event_sender.send(event);
    }

    fn next_frame_id(&self) -> RenderedWebViewFrameId {
        let id = self.next_frame_id.get();
        self.next_frame_id.set(id.saturating_add(1).max(1));
        RenderedWebViewFrameId(id)
    }
}

impl WebViewBackend for MockRenderedWebView {
    fn capabilities() -> WebViewCapabilities {
        WebViewCapabilities {
            backend_name: "mock-rendered",
            engine_name: "mock",
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
        *self.bounds.borrow_mut() = Some(bounds);
        Ok(())
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        self.visible.set(visible);
        Ok(())
    }

    fn load_html(&self, html: &str) -> Result<()> {
        *self.html.borrow_mut() = Some(html.to_owned());
        self.emit(WebViewEvent::TitleChanged {
            title: "mock html".to_owned(),
        });
        Ok(())
    }

    fn load_url(&self, url: &str) -> Result<()> {
        *self.url.borrow_mut() = Some(url.to_owned());
        self.emit(WebViewEvent::NavigationStarted {
            url: url.to_owned(),
        });
        self.emit(WebViewEvent::NavigationFinished {
            url: url.to_owned(),
        });
        Ok(())
    }

    fn evaluate_script(&self, script: &str, request_id: ScriptRequestId) -> Result<()> {
        self.scripts
            .borrow_mut()
            .push((request_id, script.to_owned()));
        self.emit(WebViewEvent::ScriptResult {
            request_id,
            value: format!("mock:{script}"),
        });
        Ok(())
    }

    fn focus(&self) -> Result<()> {
        self.emit(WebViewEvent::FocusChanged { focused: true });
        Ok(())
    }

    fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
        self.keyboard_focus_enabled.set(enabled);
        Ok(())
    }

    fn focus_parent(&self) -> Result<()> {
        self.emit(WebViewEvent::FocusChanged { focused: false });
        Ok(())
    }
}

impl RenderedWebViewBackend for MockRenderedWebView {
    fn rendered_capabilities() -> RenderedWebViewCapabilities {
        RenderedWebViewCapabilities {
            preferred_transport: RenderedWebViewFrameTransport::CpuPixels,
            supports_cpu_pixels: true,
            supports_external_textures: false,
            supports_transparency: true,
            supports_slint_input: true,
            supports_ime: true,
        }
    }

    fn resize_render_surface(&self, size: RenderedWebViewSize) -> Result<()> {
        self.sizes.borrow_mut().push(size);
        self.frames
            .borrow_mut()
            .push_back(build_mock_frame(self.next_frame_id(), size)?);
        Ok(())
    }

    fn send_input_event(&self, event: RenderedWebViewInputEvent) -> Result<()> {
        match event {
            RenderedWebViewInputEvent::FocusChanged { focused } => {
                self.emit(WebViewEvent::FocusChanged { focused });
            }
            RenderedWebViewInputEvent::PointerButton {
                state: RenderedWebViewInputState::Pressed,
                ..
            } => {
                self.emit(WebViewEvent::FocusRequested);
            }
            _ => {}
        }
        self.inputs.borrow_mut().push(event);
        Ok(())
    }

    fn next_frame(&self) -> Option<RenderedWebViewFrame> {
        self.frames.borrow_mut().pop_front()
    }
}

/// Shared controller type for the mock rendered backend.
pub type MockRenderedWebViewController = BackendRenderedWebViewController<MockRenderedWebView>;

/// Creates a mock rendered controller.
pub fn create_mock_rendered_controller() -> MockRenderedWebViewController {
    MockRenderedWebView::controller()
}

fn build_mock_frame(
    id: RenderedWebViewFrameId,
    size: RenderedWebViewSize,
) -> Result<RenderedWebViewFrame> {
    let format = RenderedWebViewPixelFormat::Rgba8Premultiplied;
    let pixel_count = (size.width as usize)
        .checked_mul(size.height as usize)
        .ok_or_else(|| {
            WebViewError::InvalidRenderedFrame("mock frame dimensions overflow".to_owned())
        })?;
    let byte_len = pixel_count
        .checked_mul(format.bytes_per_pixel())
        .ok_or_else(|| {
            WebViewError::InvalidRenderedFrame("mock frame bytes overflow".to_owned())
        })?;

    if byte_len > MAX_MOCK_FRAME_BYTES {
        return Err(WebViewError::InvalidRenderedFrame(
            "mock frame exceeds maximum test buffer size".to_owned(),
        ));
    }

    let mut bytes = Vec::with_capacity(byte_len);
    for index in 0..pixel_count {
        let x = (index % size.width as usize) as u8;
        let y = (index / size.width as usize) as u8;
        let frame = id.0 as u8;
        bytes.extend_from_slice(&[x.wrapping_mul(3), y.wrapping_mul(5), frame, 255]);
    }

    let mut frame = RenderedWebViewFrame::cpu_pixels(id, size.width, size.height, format, bytes);
    frame
        .dirty_rects
        .push(slint_webview_core::RenderedWebViewDirtyRect::new(
            0,
            0,
            size.width,
            size.height,
        ));
    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint_webview_core::{
        RenderedWebViewInputState, RenderedWebViewModifiers, RenderedWebViewPointerButton,
    };

    #[test]
    fn mock_controller_load_resize_input_and_frame() {
        let controller = create_mock_rendered_controller();

        controller.load_html("<p>fixture</p>").unwrap();
        let request_id = controller.evaluate_script("1 + 1").unwrap();
        controller
            .resize_render_surface(RenderedWebViewSize::new(4, 3, 1.0))
            .unwrap();
        controller
            .send_input_event(RenderedWebViewInputEvent::PointerButton {
                x: 1.0,
                y: 2.0,
                button: RenderedWebViewPointerButton::Primary,
                state: RenderedWebViewInputState::Pressed,
                click_count: 1,
                modifiers: RenderedWebViewModifiers::default(),
            })
            .unwrap();

        assert_eq!(
            controller.backend().last_html().as_deref(),
            Some("<p>fixture</p>")
        );
        assert_eq!(controller.backend().sizes().len(), 1);
        assert_eq!(controller.backend().inputs().len(), 1);

        let events = controller.drain_events();
        assert!(events.contains(&WebViewEvent::TitleChanged {
            title: "mock html".to_owned()
        }));
        assert!(events.contains(&WebViewEvent::FocusRequested));
        assert!(events.contains(&WebViewEvent::ScriptResult {
            request_id,
            value: "mock:1 + 1".to_owned()
        }));

        let frames = controller.drain_frames().unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].width, 4);
        assert_eq!(frames[0].height, 3);
        assert!(matches!(
            frames[0].payload,
            RenderedWebViewFramePayload::CpuPixels { .. }
        ));
    }

    #[test]
    fn mock_controller_rejects_invalid_render_size_before_backend() {
        let controller = create_mock_rendered_controller();

        assert!(matches!(
            controller.resize_render_surface(RenderedWebViewSize::new(0, 3, 1.0)),
            Err(WebViewError::InvalidRenderedSize(_))
        ));
        assert!(controller.backend().sizes().is_empty());
    }

    #[test]
    fn mock_focus_events_round_trip() {
        let controller = create_mock_rendered_controller();

        controller.focus().unwrap();
        controller.focus_parent().unwrap();

        assert_eq!(
            controller.drain_events(),
            vec![
                WebViewEvent::FocusChanged { focused: true },
                WebViewEvent::FocusChanged { focused: false },
            ]
        );
    }
}
