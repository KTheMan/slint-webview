#![deny(unsafe_code)]

use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};
use slint_webview::{
    WebViewBounds, WebViewController, WebViewError, WebViewOptions, WebViewSource,
    initialize_platform, pump_platform_events,
};

slint::slint! {
    export component App inherits Window {
        preferred-width: 900px;
        preferred-height: 600px;

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

        Rectangle {
            x: 24px;
            y: 72px;
            width: root.width - 48px;
            height: root.height - 96px;
            border-color: #93a6b2;
            border-width: 1px;
            background: #ffffff;
        }
    }
}

fn main() -> slint_webview::Result<()> {
    initialize_platform()?;

    let app = App::new().map_err(platform_error)?;
    app.show().map_err(platform_error)?;

    let options = WebViewOptions::default()
        .with_source(WebViewSource::Html(
            "<h1>Hello from the native webview</h1>".to_owned(),
        ))
        .with_bounds(WebViewBounds::new(24.0, 72.0, 852.0, 504.0))
        .with_javascript_enabled(true);

    let controller = Rc::new(WebViewController::attach(
        &app.window().window_handle(),
        options,
    )?);

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(16), {
        let controller = Rc::clone(&controller);
        move || {
            pump_platform_events();
            for event in controller.drain_events() {
                eprintln!("{event:?}");
            }
        }
    });

    let result = slint::run_event_loop().map_err(platform_error);
    drop(timer);
    result
}

fn platform_error(error: impl std::fmt::Display) -> WebViewError {
    WebViewError::Platform(error.to_string())
}
