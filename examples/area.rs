#![deny(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};
use slint_webview::{
    WebViewAreaController, WebViewAreaPolicy, WebViewAreaState, WebViewBounds, WebViewError,
    WebViewOptions, WebViewSource, initialize_platform,
};

slint::slint! {
    import { Button, CheckBox, LineEdit } from "std-widgets.slint";
    import { WebViewArea } from "../ui/webview-area.slint";

    export component AreaApp inherits Window {
        preferred-width: 900px;
        preferred-height: 600px;

        in-out property <bool> webview-requested-visible: true;
        in-out property <bool> overlay-active: false;
        in-out property <bool> shell-focus-active: false;
        in-out property <bool> webview-parked: false;
        in-out property <string> webview-status: "";

        callback sync-webview();
        callback focus-webview();
        callback release-webview-focus();

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
            placeholder-text: "native Slint input";
            edited => {
                root.shell-focus-active = true;
                root.release-webview-focus();
                root.sync-webview();
            }
            accepted => {
                root.shell-focus-active = false;
                root.sync-webview();
            }
        }

        Button {
            x: 24px;
            y: 120px;
            width: 104px;
            text: root.webview-requested-visible ? "Hide" : "Show";
            clicked => {
                root.webview-requested-visible = !root.webview-requested-visible;
                root.sync-webview();
            }
        }

        Button {
            x: 140px;
            y: 120px;
            width: 104px;
            text: "Modal";
            clicked => {
                root.overlay-active = true;
                root.sync-webview();
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
            text: root.shell-focus-active ? "shell focus active" : "webview can request focus";
            color: #40586a;
        }

        webview := WebViewArea {
            x: 276px;
            y: 72px;
            width: root.width - 300px;
            height: root.height - 96px;
            requested-visible <=> root.webview-requested-visible;
            overlay-active: root.overlay-active;
            parked: root.webview-parked;
            status-text: root.webview-status;
            focus-requested => {
                root.shell-focus-active = false;
                root.focus-webview();
                root.sync-webview();
            }
            focus-released => {
                root.release-webview-focus();
            }
        }

        if root.overlay-active : Rectangle {
            x: 300px;
            y: 160px;
            width: root.width - 360px;
            height: 180px;
            background: #ffffff;
            border-width: 1px;
            border-color: #102a3a;

            Text {
                x: 24px;
                y: 24px;
                text: "Slint modal";
                color: #102a3a;
                font-size: 24px;
                font-weight: 700;
            }

            Button {
                x: parent.width - 140px;
                y: parent.height - 56px;
                width: 112px;
                text: "Close";
                clicked => {
                    root.overlay-active = false;
                    root.sync-webview();
                }
            }
        }
    }
}

fn main() -> slint_webview::Result<()> {
    initialize_platform()?;

    let app = AreaApp::new().map_err(platform_error)?;
    app.show().map_err(platform_error)?;

    let initial_state = area_state(&app);
    let options = WebViewOptions::default()
        .with_source(WebViewSource::Html(
            "<h1>WebViewArea</h1><input placeholder='web input'><p>Native web content</p>"
                .to_owned(),
        ))
        .with_bounds(initial_state.bounds)
        .with_javascript_enabled(true);
    let controller = Rc::new(std::cell::RefCell::new(WebViewAreaController::attach(
        &app.window().window_handle(),
        options,
        initial_state,
        WebViewAreaPolicy::default(),
    )?));

    let syncing = Rc::new(Cell::new(false));
    app.on_sync_webview({
        let app = app.as_weak();
        let controller = Rc::clone(&controller);
        let syncing = Rc::clone(&syncing);
        move || {
            if syncing.replace(true) {
                return;
            }
            if let Some(app) = app.upgrade() {
                let mut controller = controller.borrow_mut();
                if let Ok(status) = controller.sync(area_state(&app)) {
                    apply_status(&app, status);
                }
            }
            syncing.set(false);
        }
    });
    app.on_focus_webview({
        let controller = Rc::clone(&controller);
        move || {
            let _ = controller.borrow_mut().focus_webview();
        }
    });
    app.on_release_webview_focus({
        let controller = Rc::clone(&controller);
        move || {
            let _ = controller.borrow_mut().release_keyboard_focus();
        }
    });

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(16), {
        let app = app.as_weak();
        let controller = Rc::clone(&controller);
        move || {
            if let Some(app) = app.upgrade() {
                let mut controller = controller.borrow_mut();
                if let Ok(events) = controller.tick(area_state(&app)) {
                    for event in events {
                        eprintln!("{event:?}");
                    }
                    apply_status(&app, controller.status());
                }
            }
        }
    });

    let result = slint::run_event_loop().map_err(platform_error);
    drop(timer);
    result
}

fn area_state(app: &AreaApp) -> WebViewAreaState {
    WebViewAreaState::new(WebViewBounds::new(276.0, 72.0, 600.0, 504.0))
        .with_requested_visible(app.get_webview_requested_visible())
        .with_overlay_active(app.get_overlay_active())
        .with_shell_focus_active(app.get_shell_focus_active())
}

fn apply_status(app: &AreaApp, status: slint_webview::WebViewAreaStatus) {
    app.set_webview_parked(status.placement.parked);
    app.set_webview_status(status_text(status).into());
}

fn status_text(status: slint_webview::WebViewAreaStatus) -> &'static str {
    if status.placement.blocked_by_overlay {
        "webview parked for Slint overlay"
    } else if status.placement.blocked_by_invalid_bounds {
        "webview waiting for valid bounds"
    } else if !status.state.requested_visible {
        "webview hidden"
    } else {
        ""
    }
}

fn platform_error(error: impl std::fmt::Display) -> WebViewError {
    WebViewError::Platform(error.to_string())
}
