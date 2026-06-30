#![deny(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};
use slint_webview::{
    RenderedWebViewFrame, RenderedWebViewFramePayload, RenderedWebViewInputEvent,
    RenderedWebViewInputState, RenderedWebViewModifiers, RenderedWebViewPointerButton,
    RenderedWebViewSize, WebViewError, WebViewEvent,
};
use slint_webview_mock::create_mock_rendered_controller;

slint::slint! {
    import { Button, CheckBox, LineEdit } from "std-widgets.slint";
    import { RenderedWebViewArea } from "../ui/rendered-webview-area.slint";

    export component RenderedMockApp inherits Window {
        preferred-width: 900px;
        preferred-height: 600px;

        in-out property <string> shell-text: "";
        in-out property <string> frame-label: "waiting for frame";
        in-out property <string> status-text: "mock backend idle";
        in-out property <bool> webview-focused: false;

        callback resize-renderer();
        callback focus-renderer();
        callback blur-renderer();
        callback pointer-pressed();

        Rectangle {
            background: #f5f8fa;
        }

        Text {
            x: 24px;
            y: 24px;
            text: "Slint shell";
            font-size: 24px;
            color: #102a3a;
        }

        LineEdit {
            x: 24px;
            y: 72px;
            width: 220px;
            text <=> root.shell-text;
            placeholder-text: "native Slint input";
            edited => {
                root.webview-focused = false;
                root.blur-renderer();
            }
        }

        Button {
            x: 24px;
            y: 120px;
            width: 104px;
            text: "Resize";
            clicked => {
                root.resize-renderer();
            }
        }

        Button {
            x: 140px;
            y: 120px;
            width: 104px;
            text: "Focus";
            clicked => {
                root.webview-focused = true;
                root.focus-renderer();
            }
        }

        CheckBox {
            x: 24px;
            y: 176px;
            text: "Slint checkbox";
        }

        Text {
            x: 24px;
            y: 224px;
            width: 220px;
            text: root.webview-focused ? "rendered area owns focus" : "native shell owns focus";
            color: #40586a;
            wrap: word-wrap;
        }

        RenderedWebViewArea {
            x: 276px;
            y: 72px;
            width: root.width - 300px;
            height: root.height - 96px;
            frame-label: root.frame-label;
            status-text: root.status-text;
            focused: root.webview-focused;
            focus-requested => {
                root.webview-focused = true;
                root.focus-renderer();
            }
            pointer-pressed => {
                root.pointer-pressed();
            }
        }
    }
}

fn main() -> slint_webview::Result<()> {
    let app = RenderedMockApp::new().map_err(platform_error)?;
    app.show().map_err(platform_error)?;

    let controller = Rc::new(create_mock_rendered_controller());
    controller.load_html("<main><h1>mock rendered webview</h1></main>")?;
    controller.resize_render_surface(RenderedWebViewSize::new(480, 320, 1.0))?;
    let large_size_next = Rc::new(Cell::new(false));

    app.on_resize_renderer({
        let controller = Rc::clone(&controller);
        let large_size_next = Rc::clone(&large_size_next);
        move || {
            let use_large_size = large_size_next.replace(!large_size_next.get());
            let next_size = if use_large_size {
                RenderedWebViewSize::new(480, 320, 1.0)
            } else {
                RenderedWebViewSize::new(320, 240, 1.0)
            };
            let _ = controller.resize_render_surface(next_size);
        }
    });
    app.on_focus_renderer({
        let controller = Rc::clone(&controller);
        move || {
            let _ = controller
                .send_input_event(RenderedWebViewInputEvent::FocusChanged { focused: true });
        }
    });
    app.on_blur_renderer({
        let controller = Rc::clone(&controller);
        move || {
            let _ = controller
                .send_input_event(RenderedWebViewInputEvent::FocusChanged { focused: false });
        }
    });
    app.on_pointer_pressed({
        let controller = Rc::clone(&controller);
        move || {
            let _ = controller.send_input_event(RenderedWebViewInputEvent::PointerButton {
                x: 24.0,
                y: 24.0,
                button: RenderedWebViewPointerButton::Primary,
                state: RenderedWebViewInputState::Pressed,
                click_count: 1,
                modifiers: RenderedWebViewModifiers::default(),
            });
        }
    });

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(16), {
        let app = app.as_weak();
        let controller = Rc::clone(&controller);
        move || {
            let Some(app) = app.upgrade() else {
                return;
            };

            for event in controller.drain_events() {
                apply_event(&app, event);
            }
            if let Ok(frames) = controller.drain_frames() {
                for frame in frames {
                    app.set_frame_label(frame_label(&frame).into());
                }
            }
        }
    });

    let result = slint::run_event_loop().map_err(platform_error);
    drop(timer);
    result
}

fn apply_event(app: &RenderedMockApp, event: WebViewEvent) {
    match event {
        WebViewEvent::TitleChanged { title } => {
            app.set_status_text(format!("loaded: {title}").into());
        }
        WebViewEvent::FocusChanged { focused } => {
            app.set_webview_focused(focused);
            app.set_status_text(if focused {
                "mock backend focused".into()
            } else {
                "mock backend blurred".into()
            });
        }
        WebViewEvent::FocusRequested => {
            app.set_webview_focused(true);
            app.set_status_text("mock backend requested focus".into());
        }
        _ => {}
    }
}

fn frame_label(frame: &RenderedWebViewFrame) -> String {
    let transport = match &frame.payload {
        RenderedWebViewFramePayload::CpuPixels { .. } => "CPU pixels",
        RenderedWebViewFramePayload::ExternalTexture { .. } => "external texture",
    };
    format!(
        "frame {}: {}x{} {transport}",
        frame.id.0, frame.width, frame.height
    )
}

fn platform_error(error: impl std::fmt::Display) -> WebViewError {
    WebViewError::Platform(error.to_string())
}
