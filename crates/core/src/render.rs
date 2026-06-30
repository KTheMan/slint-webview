use serde::{Deserialize, Serialize};

use crate::Result;

/// Pixel transport used by a Slint-owned rendered backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderedWebViewFrameTransport {
    /// Backend sends CPU pixel buffers to the host.
    CpuPixels,
    /// Backend exposes an external GPU texture handle or identifier.
    ExternalTexture,
}

/// CPU or texture pixel format for rendered webview frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderedWebViewPixelFormat {
    /// Four bytes per pixel in BGRA order with premultiplied alpha.
    Bgra8Premultiplied,
    /// Four bytes per pixel in RGBA order with premultiplied alpha.
    Rgba8Premultiplied,
    /// Four bytes per pixel in RGBA order without premultiplied alpha.
    Rgba8Unpremultiplied,
}

impl RenderedWebViewPixelFormat {
    /// Returns the number of bytes used by one pixel in this format.
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Bgra8Premultiplied | Self::Rgba8Premultiplied | Self::Rgba8Unpremultiplied => 4,
        }
    }
}

/// Graphics API that owns an external webview texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderedWebViewTextureApi {
    /// Direct3D 11 texture.
    D3d11,
    /// Direct3D 12 texture.
    D3d12,
    /// Metal texture.
    Metal,
    /// OpenGL texture.
    OpenGl,
    /// Vulkan image.
    Vulkan,
    /// Platform-specific texture handle not covered by a stable variant yet.
    Platform,
}

/// Integer pixel size for a rendered webview surface.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderedWebViewSize {
    /// Surface width in physical pixels.
    pub width: u32,
    /// Surface height in physical pixels.
    pub height: u32,
    /// Device scale factor used when translating from Slint logical pixels.
    pub scale_factor: f64,
}

impl RenderedWebViewSize {
    /// Creates a rendered surface size.
    pub const fn new(width: u32, height: u32, scale_factor: f64) -> Self {
        Self {
            width,
            height,
            scale_factor,
        }
    }

    /// Returns true when width, height, and scale are usable for rendering.
    pub fn is_valid(self) -> bool {
        self.width > 0
            && self.height > 0
            && self.scale_factor.is_finite()
            && self.scale_factor > 0.0
    }
}

/// Dirty rectangle in physical pixels inside a rendered frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedWebViewDirtyRect {
    /// Left pixel coordinate.
    pub x: u32,
    /// Top pixel coordinate.
    pub y: u32,
    /// Dirty width in pixels.
    pub width: u32,
    /// Dirty height in pixels.
    pub height: u32,
}

impl RenderedWebViewDirtyRect {
    /// Creates a dirty rectangle in physical pixels.
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Monotonic identifier for rendered frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderedWebViewFrameId(pub u64);

/// Pixel payload for a rendered webview frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderedWebViewFramePayload {
    /// CPU pixel data owned by the frame.
    CpuPixels {
        /// Pixel bytes.
        bytes: Vec<u8>,
        /// Number of bytes between the start of two adjacent rows.
        stride: usize,
        /// Pixel format.
        format: RenderedWebViewPixelFormat,
    },
    /// External texture produced by the backend.
    ExternalTexture {
        /// Opaque backend texture identifier.
        texture_id: u64,
        /// Graphics API that owns the texture.
        api: RenderedWebViewTextureApi,
        /// Pixel format.
        format: RenderedWebViewPixelFormat,
    },
}

impl RenderedWebViewFramePayload {
    /// Returns the pixel format for this payload.
    pub const fn format(&self) -> RenderedWebViewPixelFormat {
        match self {
            Self::CpuPixels { format, .. } | Self::ExternalTexture { format, .. } => *format,
        }
    }

    /// Returns the frame transport used by this payload.
    pub const fn transport(&self) -> RenderedWebViewFrameTransport {
        match self {
            Self::CpuPixels { .. } => RenderedWebViewFrameTransport::CpuPixels,
            Self::ExternalTexture { .. } => RenderedWebViewFrameTransport::ExternalTexture,
        }
    }
}

/// Rendered webview frame produced by a Slint-owned backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedWebViewFrame {
    /// Frame identifier.
    pub id: RenderedWebViewFrameId,
    /// Surface width in physical pixels.
    pub width: u32,
    /// Surface height in physical pixels.
    pub height: u32,
    /// Dirty rectangles changed by this frame. Empty means the whole frame.
    pub dirty_rects: Vec<RenderedWebViewDirtyRect>,
    /// Pixel or texture payload.
    pub payload: RenderedWebViewFramePayload,
}

impl RenderedWebViewFrame {
    /// Creates a frame from owned CPU pixels.
    pub fn cpu_pixels(
        id: RenderedWebViewFrameId,
        width: u32,
        height: u32,
        format: RenderedWebViewPixelFormat,
        bytes: Vec<u8>,
    ) -> Self {
        let stride = width as usize * format.bytes_per_pixel();
        Self {
            id,
            width,
            height,
            dirty_rects: Vec::new(),
            payload: RenderedWebViewFramePayload::CpuPixels {
                bytes,
                stride,
                format,
            },
        }
    }

    /// Returns the minimum number of bytes required for a tightly packed CPU
    /// frame of this size and format.
    pub fn minimum_cpu_byte_len(
        width: u32,
        height: u32,
        format: RenderedWebViewPixelFormat,
    ) -> usize {
        width as usize * height as usize * format.bytes_per_pixel()
    }

    /// Returns true when this frame has positive dimensions.
    pub const fn has_valid_dimensions(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// Keyboard modifier state for rendered backend input events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedWebViewModifiers {
    /// Shift is pressed.
    pub shift: bool,
    /// Control is pressed.
    pub control: bool,
    /// Alt or Option is pressed.
    pub alt: bool,
    /// Meta, Command, or Windows key is pressed.
    pub meta: bool,
}

/// Button or key transition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderedWebViewInputState {
    /// Input was pressed.
    Pressed,
    /// Input was released.
    Released,
}

/// Pointer button used by rendered backend input events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderedWebViewPointerButton {
    /// Primary pointer button.
    Primary,
    /// Secondary pointer button.
    Secondary,
    /// Middle pointer button.
    Middle,
    /// Other platform button number.
    Other(u16),
}

/// Input event sent from Slint into a rendered webview backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderedWebViewInputEvent {
    /// Pointer moved over the rendered surface.
    PointerMoved {
        /// Logical x coordinate within the webview area.
        x: f64,
        /// Logical y coordinate within the webview area.
        y: f64,
        /// Active modifiers.
        modifiers: RenderedWebViewModifiers,
    },
    /// Pointer button changed state.
    PointerButton {
        /// Logical x coordinate within the webview area.
        x: f64,
        /// Logical y coordinate within the webview area.
        y: f64,
        /// Button that changed state.
        button: RenderedWebViewPointerButton,
        /// Button state.
        state: RenderedWebViewInputState,
        /// Consecutive click count reported by the host.
        click_count: u8,
        /// Active modifiers.
        modifiers: RenderedWebViewModifiers,
    },
    /// Wheel or trackpad scroll input.
    Wheel {
        /// Logical x coordinate within the webview area.
        x: f64,
        /// Logical y coordinate within the webview area.
        y: f64,
        /// Horizontal wheel delta.
        delta_x: f64,
        /// Vertical wheel delta.
        delta_y: f64,
        /// Active modifiers.
        modifiers: RenderedWebViewModifiers,
    },
    /// Keyboard input.
    Keyboard {
        /// Backend-neutral key name.
        key: String,
        /// Text produced by the key, when applicable.
        text: Option<String>,
        /// Key state.
        state: RenderedWebViewInputState,
        /// Active modifiers.
        modifiers: RenderedWebViewModifiers,
    },
    /// Text committed by the platform input method editor.
    ImeCommit {
        /// Committed UTF-8 text.
        text: String,
    },
    /// Host focus changed.
    FocusChanged {
        /// True when the rendered webview owns focus.
        focused: bool,
    },
}

/// Capabilities specific to a Slint-owned rendered backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedWebViewCapabilities {
    /// Preferred frame transport for this backend.
    pub preferred_transport: RenderedWebViewFrameTransport,
    /// Whether CPU pixel frames are supported.
    pub supports_cpu_pixels: bool,
    /// Whether external texture frames are supported.
    pub supports_external_textures: bool,
    /// Whether transparent frame output is supported.
    pub supports_transparency: bool,
    /// Whether keyboard and pointer input can be translated from Slint events.
    pub supports_slint_input: bool,
    /// Whether input method editor text composition can be forwarded.
    pub supports_ime: bool,
}

impl RenderedWebViewCapabilities {
    /// Planned rendered capabilities for Servo.
    pub const fn servo_texture() -> Self {
        Self {
            preferred_transport: RenderedWebViewFrameTransport::ExternalTexture,
            supports_cpu_pixels: true,
            supports_external_textures: true,
            supports_transparency: true,
            supports_slint_input: true,
            supports_ime: true,
        }
    }

    /// Planned rendered capabilities for CEF.
    pub const fn cef_offscreen() -> Self {
        Self {
            preferred_transport: RenderedWebViewFrameTransport::CpuPixels,
            supports_cpu_pixels: true,
            supports_external_textures: true,
            supports_transparency: true,
            supports_slint_input: true,
            supports_ime: true,
        }
    }
}

/// Additional contract implemented by Slint-owned rendered backends.
///
/// Servo and CEF backends are expected to implement both [`crate::WebViewBackend`]
/// for browser operations and this trait for frame production and input routing.
pub trait RenderedWebViewBackend {
    /// Returns rendered-backend capabilities.
    fn rendered_capabilities() -> RenderedWebViewCapabilities
    where
        Self: Sized;

    /// Resizes the rendered surface in physical pixels.
    fn resize_render_surface(&self, size: RenderedWebViewSize) -> Result<()>;

    /// Sends a Slint-originated input event to the rendered webview.
    fn send_input_event(&self, event: RenderedWebViewInputEvent) -> Result<()>;

    /// Returns the next produced frame, if one is ready.
    fn next_frame(&self) -> Option<RenderedWebViewFrame>;
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[test]
    fn rendered_size_requires_positive_dimensions_and_scale() {
        assert!(RenderedWebViewSize::new(100, 80, 1.5).is_valid());
        assert!(!RenderedWebViewSize::new(0, 80, 1.5).is_valid());
        assert!(!RenderedWebViewSize::new(100, 80, 0.0).is_valid());
    }

    #[test]
    fn cpu_frame_reports_expected_transport_and_format() {
        let frame = RenderedWebViewFrame::cpu_pixels(
            RenderedWebViewFrameId(7),
            2,
            2,
            RenderedWebViewPixelFormat::Bgra8Premultiplied,
            vec![0; 16],
        );

        assert_eq!(
            frame.payload.transport(),
            RenderedWebViewFrameTransport::CpuPixels
        );
        assert_eq!(
            frame.payload.format(),
            RenderedWebViewPixelFormat::Bgra8Premultiplied
        );
        assert_eq!(
            RenderedWebViewFrame::minimum_cpu_byte_len(
                2,
                2,
                RenderedWebViewPixelFormat::Bgra8Premultiplied
            ),
            16
        );
    }

    #[derive(Default)]
    struct RecordingRenderedBackend {
        inputs: RefCell<Vec<RenderedWebViewInputEvent>>,
        frames: RefCell<Vec<RenderedWebViewFrame>>,
        sizes: RefCell<Vec<RenderedWebViewSize>>,
    }

    impl RenderedWebViewBackend for RecordingRenderedBackend {
        fn rendered_capabilities() -> RenderedWebViewCapabilities {
            RenderedWebViewCapabilities::cef_offscreen()
        }

        fn resize_render_surface(&self, size: RenderedWebViewSize) -> Result<()> {
            self.sizes.borrow_mut().push(size);
            Ok(())
        }

        fn send_input_event(&self, event: RenderedWebViewInputEvent) -> Result<()> {
            self.inputs.borrow_mut().push(event);
            Ok(())
        }

        fn next_frame(&self) -> Option<RenderedWebViewFrame> {
            self.frames.borrow_mut().pop()
        }
    }

    #[test]
    fn rendered_backend_contract_routes_resize_input_and_frames() {
        let backend = RecordingRenderedBackend::default();
        let frame = RenderedWebViewFrame::cpu_pixels(
            RenderedWebViewFrameId(1),
            1,
            1,
            RenderedWebViewPixelFormat::Rgba8Premultiplied,
            vec![255; 4],
        );
        backend.frames.borrow_mut().push(frame.clone());

        backend
            .resize_render_surface(RenderedWebViewSize::new(1, 1, 1.0))
            .unwrap();
        backend
            .send_input_event(RenderedWebViewInputEvent::FocusChanged { focused: true })
            .unwrap();

        assert_eq!(backend.sizes.borrow().len(), 1);
        assert_eq!(backend.inputs.borrow().len(), 1);
        assert_eq!(backend.next_frame(), Some(frame));
        assert!(backend.next_frame().is_none());
    }
}
