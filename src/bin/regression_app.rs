#![deny(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::{ComponentHandle, Timer, TimerMode};
use slint_webview::{
    NavigationPolicy, WebViewBounds, WebViewController, WebViewEvent, WebViewOptions,
    WebViewSource, fixture, initialize_platform, pump_platform_events,
};

slint::slint! {
    import { Button, CheckBox, LineEdit } from "std-widgets.slint";

    export component App inherits Window {
        preferred-width: 1180px;
        preferred-height: 760px;
        min-width: 900px;
        min-height: 560px;

        in-out property <string> status: "starting";
        in-out property <string> shell-text: "";
        in-out property <bool> shell-input-focused: false;
        in-out property <bool> webview-requested-visible: true;
        in-out property <bool> webview-visible: true;
        in-out property <bool> modal-open: false;
        in-out property <int> shell-clicks: 0;
        callback reload-webview();
        callback run-dom-probe();
        callback toggle-webview();
        callback focus-shell();
        callback release-shell-focus();
        callback repair-native-focus();
        callback open-modal();
        callback close-modal();

        release-shell-focus => {
            root.shell-input-focused = false;
            focus-sink.focus();
        }

        Rectangle {
            background: #eef3f6;
        }

        focus-sink := FocusScope {
            x: -4px;
            y: -4px;
            width: 1px;
            height: 1px;
        }

        Rectangle {
            x: 0px;
            y: 0px;
            width: root.width;
            height: 56px;
            background: #102a3a;
        }

        Text {
            x: 16px;
            y: 15px;
            text: "slint-webview regression";
            color: #ffffff;
            font-size: 20px;
            font-weight: 700;
        }

        Button {
            x: 300px;
            y: 10px;
            width: 120px;
            height: 36px;
            text: "Reload";
            clicked => { root.reload-webview(); }
        }

        Button {
            x: 430px;
            y: 10px;
            width: 120px;
            height: 36px;
            text: "Probe";
            clicked => { root.run-dom-probe(); }
        }

        Button {
            x: 560px;
            y: 10px;
            width: 132px;
            height: 36px;
            text: root.webview-requested-visible ? "Hide WebView" : "Show WebView";
            clicked => { root.toggle-webview(); }
        }

        Button {
            x: 704px;
            y: 10px;
            width: 110px;
            height: 36px;
            text: "Modal";
            clicked => { root.open-modal(); }
        }

        Rectangle {
            x: 0px;
            y: 56px;
            width: 284px;
            height: root.height - 90px;
            background: #ffffff;
            border-color: #c9d5dc;
            border-width: 1px;
        }

        Text {
            x: 16px;
            y: 76px;
            width: 250px;
            text: "Slint shell controls";
            color: #163246;
            font-size: 18px;
            font-weight: 700;
        }

        shell-input := LineEdit {
            x: 16px;
            y: 114px;
            width: 250px;
            height: 36px;
            placeholder-text: "type after using webview";
            text <=> root.shell-text;
            edited(text) => {
                root.status = "shell text: " + text;
            }
            changed has-focus => {
                root.shell-input-focused = self.has-focus;
                if self.has-focus {
                    root.repair-native-focus();
                }
            }
        }

        Button {
            x: 16px;
            y: 164px;
            width: 120px;
            height: 36px;
            text: "Shell Click";
            clicked => {
                root.shell-clicks += 1;
                root.status = "shell click " + root.shell-clicks;
            }
        }

        Button {
            x: 146px;
            y: 164px;
            width: 120px;
            height: 36px;
            text: "Focus Shell";
            clicked => {
                root.focus-shell();
                shell-input.focus();
            }
        }

        CheckBox {
            x: 16px;
            y: 216px;
            width: 250px;
            height: 32px;
            text: "Slint checkbox still works";
        }

        Text {
            x: 16px;
            y: 270px;
            width: 250px;
            height: 120px;
            text: "The webview is a native child surface. The shell around it must keep working during focus, resize, modal, and messaging checks.";
            color: #41586a;
            wrap: word-wrap;
        }

        Rectangle {
            x: 300px;
            y: 64px;
            width: root.width - 316px;
            height: root.height - 104px;
            background: root.webview-visible ? #172635 : #d6e4eb;
            border-color: root.webview-visible ? #29c4b6 : #93a6b2;
            border-width: 2px;
        }

        Text {
            x: 318px;
            y: 82px;
            text: root.webview-visible ? "native webview surface" : "webview hidden";
            color: root.webview-visible ? #f0fbff : #41586a;
            font-size: 16px;
            font-weight: 700;
        }

        Rectangle {
            x: 0px;
            y: root.height - 34px;
            width: root.width;
            height: 34px;
            background: #102a3a;
        }

        Text {
            x: 16px;
            y: root.height - 26px;
            width: root.width - 32px;
            height: 22px;
            text: root.status;
            color: #ffffff;
            font-size: 13px;
            overflow: elide;
        }

        if root.modal-open: Rectangle {
            x: 240px;
            y: 120px;
            width: root.width - 480px;
            height: 240px;
            background: #ffffff;
            border-color: #102a3a;
            border-width: 2px;

            Text {
                x: 24px;
                y: 24px;
                width: parent.width - 48px;
                text: "Modal overlay policy";
                color: #102a3a;
                font-size: 22px;
                font-weight: 700;
            }

            Text {
                x: 24px;
                y: 70px;
                width: parent.width - 48px;
                text: "Tier 1 native child webviews are hidden while this modal is open, so Slint UI remains visually and interactively correct.";
                color: #41586a;
                wrap: word-wrap;
            }

            Button {
                x: parent.width - 144px;
                y: parent.height - 58px;
                width: 120px;
                height: 36px;
                text: "Close";
                clicked => { root.close-modal(); }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    let hold_seconds = std::env::args()
        .find_map(|arg| {
            arg.strip_prefix("--hold-seconds=")
                .and_then(|value| value.parse::<u64>().ok())
        })
        .unwrap_or(0);
    initialize_platform()?;

    let app = App::new()?;
    app.set_status("showing Slint shell".into());
    app.show()?;

    let webview = Rc::new(RefCell::new(None::<WebViewController>));
    let latest_bounds = Rc::new(RefCell::new(WebViewBounds::default()));
    let requested_visible = Rc::new(Cell::new(true));
    let last_probe_at = Rc::new(RefCell::new(None::<Instant>));
    let last_focus_repair_at = Rc::new(RefCell::new(None::<Instant>));
    let parking_flush_until = Rc::new(RefCell::new(None::<Instant>));
    let composition_flush_until = Rc::new(RefCell::new(None::<Instant>));
    let initial_reveal_pending = Rc::new(Cell::new(false));
    let webview_keyboard_claimed = Rc::new(Cell::new(false));
    let last_native_bounds_sync_at = Rc::new(RefCell::new(None::<Instant>));
    let smoke_succeeded_at = Rc::new(RefCell::new(None::<Instant>));
    let smoke_started = Instant::now();
    let attach_delay = if cfg!(target_os = "linux") {
        Duration::from_millis(3000)
    } else {
        Duration::ZERO
    };

    app.on_toggle_webview({
        let app = app.as_weak();
        let webview = Rc::clone(&webview);
        let latest_bounds = Rc::clone(&latest_bounds);
        let requested_visible = Rc::clone(&requested_visible);
        let parking_flush_until = Rc::clone(&parking_flush_until);
        move || {
            if let Some(app) = app.upgrade() {
                let next = !requested_visible.get();
                requested_visible.set(next);
                app.set_webview_requested_visible(next);
                sync_webview_visibility(
                    &app,
                    webview.borrow().as_ref(),
                    *latest_bounds.borrow(),
                    next,
                    "toggle",
                );
                update_parking_flush(&app, &parking_flush_until);
            }
        }
    });

    app.on_focus_shell({
        let app = app.as_weak();
        let webview = Rc::clone(&webview);
        let webview_keyboard_claimed = Rc::clone(&webview_keyboard_claimed);
        move || {
            webview_keyboard_claimed.set(false);
            if let Some(app) = app.upgrade() {
                if let Err(error) = repair_native_focus(webview.borrow().as_ref()) {
                    app.set_status(format!("focus parent failed: {error}").into());
                    return;
                }
                app.set_status("shell input focus requested".into());
            }
        }
    });

    app.on_repair_native_focus({
        let app = app.as_weak();
        let webview = Rc::clone(&webview);
        let webview_keyboard_claimed = Rc::clone(&webview_keyboard_claimed);
        move || {
            webview_keyboard_claimed.set(false);
            if let Err(error) = repair_native_focus(webview.borrow().as_ref())
                && let Some(app) = app.upgrade()
            {
                app.set_status(format!("native focus repair failed: {error}").into());
            }
        }
    });

    app.on_open_modal({
        let app = app.as_weak();
        let webview = Rc::clone(&webview);
        let latest_bounds = Rc::clone(&latest_bounds);
        let requested_visible = Rc::clone(&requested_visible);
        let parking_flush_until = Rc::clone(&parking_flush_until);
        move || {
            if let Some(app) = app.upgrade() {
                app.set_modal_open(true);
                sync_webview_visibility(
                    &app,
                    webview.borrow().as_ref(),
                    *latest_bounds.borrow(),
                    requested_visible.get(),
                    "modal opened",
                );
                update_parking_flush(&app, &parking_flush_until);
            }
        }
    });

    app.on_close_modal({
        let app = app.as_weak();
        let webview = Rc::clone(&webview);
        let latest_bounds = Rc::clone(&latest_bounds);
        let requested_visible = Rc::clone(&requested_visible);
        let parking_flush_until = Rc::clone(&parking_flush_until);
        move || {
            if let Some(app) = app.upgrade() {
                app.set_modal_open(false);
                sync_webview_visibility(
                    &app,
                    webview.borrow().as_ref(),
                    *latest_bounds.borrow(),
                    requested_visible.get(),
                    "modal closed",
                );
                update_parking_flush(&app, &parking_flush_until);
            }
        }
    });

    app.on_reload_webview({
        let app = app.as_weak();
        let webview = Rc::clone(&webview);
        move || {
            if let Some(webview) = webview.borrow().as_ref() {
                match webview.load_html(fixture::fixture_html()) {
                    Ok(()) => {
                        if let Some(app) = app.upgrade() {
                            app.set_status("fixture reloaded".into());
                        }
                    }
                    Err(error) => {
                        if let Some(app) = app.upgrade() {
                            app.set_status(format!("reload failed: {error}").into());
                        }
                    }
                }
            }
        }
    });

    app.on_run_dom_probe({
        let app = app.as_weak();
        let webview = Rc::clone(&webview);
        move || {
            if let Some(webview) = webview.borrow().as_ref()
                && let Err(error) = webview.evaluate_script(fixture::fixture_state_script())
                && let Some(app) = app.upgrade()
            {
                app.set_status(format!("probe failed: {error}").into());
            }
        }
    });

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(100), {
        let app = app.as_weak();
        let webview = Rc::clone(&webview);
        let latest_bounds = Rc::clone(&latest_bounds);
        let requested_visible = Rc::clone(&requested_visible);
        let last_probe_at = Rc::clone(&last_probe_at);
        let last_focus_repair_at = Rc::clone(&last_focus_repair_at);
        let parking_flush_until = Rc::clone(&parking_flush_until);
        let composition_flush_until = Rc::clone(&composition_flush_until);
        let initial_reveal_pending = Rc::clone(&initial_reveal_pending);
        let webview_keyboard_claimed = Rc::clone(&webview_keyboard_claimed);
        let last_native_bounds_sync_at = Rc::clone(&last_native_bounds_sync_at);
        let smoke_succeeded_at = Rc::clone(&smoke_succeeded_at);
        move || {
            pump_platform_events();

            let Some(app) = app.upgrade() else {
                let _ = slint::quit_event_loop();
                return;
            };

            if let Some(native_webview) = webview.borrow().as_ref() {
                for event in native_webview.drain_events() {
                    if is_webview_activity_event(&event) {
                        webview_keyboard_claimed.set(true);
                        app.invoke_release_shell_focus();
                    }
                    handle_event(&app, &event, smoke);
                    match &event {
                        WebViewEvent::FocusRequested => {
                            webview_keyboard_claimed.set(true);
                            app.invoke_release_shell_focus();
                            let _ = native_webview.set_keyboard_focus_enabled(true);
                            let _ = native_webview.focus();
                        }
                        WebViewEvent::FocusChanged { focused } => {
                            if *focused && webview_keyboard_claimed.get() {
                                let _ = native_webview.set_keyboard_focus_enabled(true);
                            } else if *focused {
                                let _ = repair_native_focus(Some(native_webview));
                            } else {
                                webview_keyboard_claimed.set(false);
                                if app.get_shell_input_focused() {
                                    let _ = native_webview.set_keyboard_focus_enabled(false);
                                }
                            }
                        }
                        _ => {}
                    }
                    if initial_reveal_pending.get() && initial_webview_ready_to_reveal(&event) {
                        initial_reveal_pending.set(false);
                        sync_webview_visibility(
                            &app,
                            Some(native_webview),
                            *latest_bounds.borrow(),
                            requested_visible.get(),
                            "initial content ready",
                        );
                    }
                    if smoke
                        && let WebViewEvent::ScriptResult { value, .. } = &event
                        && value.contains(fixture::FIXTURE_READY_TOKEN)
                    {
                        println!("smoke-ok: {value}");
                        *smoke_succeeded_at.borrow_mut() = Some(Instant::now());
                        if hold_seconds == 0 {
                            let _ = slint::quit_event_loop();
                            return;
                        }
                    }
                }
            }

            if app.get_shell_input_focused()
                && !webview_keyboard_claimed.get()
                && last_focus_repair_at
                    .borrow()
                    .is_none_or(|last_repair| last_repair.elapsed() > Duration::from_millis(50))
            {
                *last_focus_repair_at.borrow_mut() = Some(Instant::now());
                let _ = repair_native_focus(webview.borrow().as_ref());
            }

            if !app.get_webview_visible()
                && parking_flush_until
                    .borrow()
                    .as_ref()
                    .is_some_and(|deadline| Instant::now() < *deadline)
            {
                if let Some(webview) = webview.borrow().as_ref() {
                    let _ = park_webview(&app, webview);
                }
                flush_native_composition(&app);
            }

            let should_flush_composition = composition_flush_until
                .borrow()
                .as_ref()
                .is_some_and(|deadline| Instant::now() < *deadline);
            if should_flush_composition {
                flush_native_composition(&app);
            } else {
                *composition_flush_until.borrow_mut() = None;
            }

            let bounds = regression_bounds(&app);
            if *latest_bounds.borrow() != bounds {
                *latest_bounds.borrow_mut() = bounds;
                if app.get_webview_visible()
                    && !initial_reveal_pending.get()
                    && let Some(webview) = webview.borrow().as_ref()
                {
                    match webview.set_bounds(native_bounds(&app, bounds)) {
                        Ok(()) => schedule_composition_flush(&app, &composition_flush_until),
                        Err(error) => app.set_status(format!("bounds sync failed: {error}").into()),
                    }
                }
                *last_native_bounds_sync_at.borrow_mut() = Some(Instant::now());
            }

            if cfg!(target_os = "linux")
                && app.get_webview_visible()
                && !initial_reveal_pending.get()
                && last_native_bounds_sync_at
                    .borrow()
                    .is_none_or(|last_sync| last_sync.elapsed() > Duration::from_millis(250))
            {
                *last_native_bounds_sync_at.borrow_mut() = Some(Instant::now());
                if let Some(webview) = webview.borrow().as_ref() {
                    match webview.set_bounds(native_bounds(&app, *latest_bounds.borrow())) {
                        Ok(()) => flush_native_composition(&app),
                        Err(error) => {
                            app.set_status(format!("linux bounds resync failed: {error}").into());
                        }
                    }
                }
            }

            if webview.borrow().is_none() {
                if smoke_started.elapsed() < attach_delay {
                    app.set_status("waiting for native window map before webview attach".into());
                    return;
                }

                let warm_initial_linux_webview =
                    cfg!(target_os = "linux") && should_show_webview(&app, requested_visible.get());
                let initial_bounds = if warm_initial_linux_webview {
                    native_bounds(&app, parked_webview_bounds(&app))
                } else {
                    native_bounds(&app, bounds)
                };

                let options = WebViewOptions {
                    source: WebViewSource::Html(fixture::fixture_html().to_owned()),
                    bounds: initial_bounds,
                    javascript_enabled: true,
                    clipboard_enabled: true,
                    visible: should_show_webview(&app, requested_visible.get()),
                    navigation_policy: NavigationPolicy::BlockSchemes(vec![
                        "slint-blocked".to_owned(),
                    ]),
                    ..WebViewOptions::default()
                };

                let handle = app.window().window_handle();
                match WebViewController::attach(&handle, options) {
                    Ok(native) => {
                        app.set_status(
                            format!(
                                "webview attached via {} / {:?}",
                                WebViewController::capabilities().engine_name,
                                bounds
                            )
                            .into(),
                        );
                        if smoke {
                            println!(
                                "smoke-attached: {} {:?}",
                                WebViewController::capabilities().engine_name,
                                bounds
                            );
                        }
                        *webview.borrow_mut() = Some(native);
                        if warm_initial_linux_webview {
                            initial_reveal_pending.set(true);
                            app.set_webview_visible(true);
                            app.set_status("warming native webview off-window".into());
                            flush_native_composition(&app);
                        } else {
                            sync_webview_visibility(
                                &app,
                                webview.borrow().as_ref(),
                                bounds,
                                requested_visible.get(),
                                "attached",
                            );
                        }
                    }
                    Err(error) => {
                        app.set_status(format!("waiting for native webview: {error}").into());
                    }
                }
            }

            let should_probe = smoke
                && smoke_succeeded_at.borrow().is_none()
                && webview.borrow().is_some()
                && smoke_started.elapsed() > attach_delay + Duration::from_millis(500)
                && last_probe_at
                    .borrow()
                    .is_none_or(|last_probe| last_probe.elapsed() > Duration::from_secs(1));

            if should_probe {
                *last_probe_at.borrow_mut() = Some(Instant::now());
                if let Some(webview) = webview.borrow().as_ref() {
                    let _ = webview.evaluate_script(fixture::fixture_state_script());
                }
            }

            if smoke
                && smoke_succeeded_at.borrow().is_none()
                && smoke_started.elapsed() > Duration::from_secs(12)
            {
                eprintln!("smoke-timeout");
                std::process::exit(2);
            }

            if let Some(success_at) = *smoke_succeeded_at.borrow()
                && success_at.elapsed() > Duration::from_secs(hold_seconds)
            {
                let _ = slint::quit_event_loop();
            }
        }
    });

    let event_loop_result = slint::run_event_loop();
    drop(timer);

    if let Err(error) = event_loop_result {
        if smoke && smoke_succeeded_at.borrow().is_some() {
            eprintln!("smoke-exit-warning: {error}");
            return Ok(());
        }
        return Err(error.into());
    }

    Ok(())
}

fn should_show_webview(app: &App, requested_visible: bool) -> bool {
    requested_visible && !app.get_modal_open()
}

fn sync_webview_visibility(
    app: &App,
    webview: Option<&WebViewController>,
    bounds: WebViewBounds,
    requested_visible: bool,
    reason: &str,
) {
    let visible = should_show_webview(app, requested_visible);
    app.set_webview_visible(visible);
    app.set_webview_requested_visible(requested_visible);

    let result = webview.map(|webview| {
        if visible {
            webview
                .set_bounds(native_bounds(app, bounds))
                .and_then(|()| webview.set_visible(true))
        } else {
            park_webview(app, webview)
        }
    });

    if let Some(Err(error)) = result {
        app.set_status(format!("{reason}: webview visibility failed: {error}").into());
        return;
    }

    let status = if visible {
        format!("{reason}: webview shown")
    } else if app.get_modal_open() {
        format!("{reason}: webview parked for modal")
    } else {
        format!("{reason}: webview hidden")
    };
    app.set_status(status.into());
    flush_native_composition(app);
}

fn park_webview(app: &App, webview: &WebViewController) -> slint_webview::Result<()> {
    let _ = webview.focus_parent();
    webview.set_bounds(native_bounds(app, parked_webview_bounds(app)))?;

    if cfg!(target_os = "linux") {
        pump_platform_events();
        Ok(())
    } else {
        webview.set_visible(false)
    }
}

fn update_parking_flush(app: &App, parking_flush_until: &RefCell<Option<Instant>>) {
    *parking_flush_until.borrow_mut() = if app.get_webview_visible() {
        None
    } else {
        Some(Instant::now() + Duration::from_secs(2))
    };
    flush_native_composition(app);
}

fn schedule_composition_flush(app: &App, composition_flush_until: &RefCell<Option<Instant>>) {
    if cfg!(target_os = "linux") {
        *composition_flush_until.borrow_mut() = Some(Instant::now() + Duration::from_millis(900));
    }
    flush_native_composition(app);
}

fn flush_native_composition(app: &App) {
    app.window().request_redraw();
    pump_platform_events();
    pump_platform_events();
}

fn repair_native_focus(webview: Option<&WebViewController>) -> slint_webview::Result<()> {
    if let Some(webview) = webview {
        webview.set_keyboard_focus_enabled(false)?;
        webview.focus_parent()?;
        pump_platform_events();
    }
    Ok(())
}

fn initial_webview_ready_to_reveal(event: &WebViewEvent) -> bool {
    match event {
        WebViewEvent::NavigationFinished { .. } => true,
        WebViewEvent::TitleChanged { title } => title.contains(fixture::FIXTURE_READY_TOKEN),
        _ => false,
    }
}

fn native_bounds(app: &App, bounds: WebViewBounds) -> WebViewBounds {
    if cfg!(target_os = "linux") {
        let scale = f64::from(app.window().scale_factor()).max(1.0);
        WebViewBounds::new(
            bounds.x * scale,
            bounds.y * scale,
            bounds.width * scale,
            bounds.height * scale,
        )
    } else {
        bounds
    }
}

fn parked_webview_bounds(app: &App) -> WebViewBounds {
    let window = app.window();
    let size = window.size();
    let scale = f64::from(window.scale_factor()).max(1.0);
    let logical_width = f64::from(size.width) / scale;
    let logical_height = f64::from(size.height) / scale;

    WebViewBounds::new(logical_width + 8.0, logical_height + 8.0, 1.0, 1.0)
}

fn handle_event(app: &App, event: &WebViewEvent, smoke: bool) {
    match event {
        WebViewEvent::NavigationStarted { url } => app.set_status(format!("loading {url}").into()),
        WebViewEvent::NavigationFinished { url } => app.set_status(format!("loaded {url}").into()),
        WebViewEvent::NavigationRequested { url, decision } => {
            app.set_status(format!("navigation requested {url}: {decision:?}").into())
        }
        WebViewEvent::IpcMessage { body, .. } => app.set_status(format!("ipc {body}").into()),
        WebViewEvent::TitleChanged { title } => app.set_status(format!("title {title}").into()),
        WebViewEvent::FocusRequested => app.set_status("webview focus requested by click".into()),
        WebViewEvent::FocusChanged { focused } => {
            app.set_status(format!("webview native focus: {focused}").into())
        }
        WebViewEvent::ScriptResult { value, .. } => {
            if smoke {
                app.set_status("smoke probe returned fixture state".into());
            } else {
                app.set_status(format!("probe {value}").into());
            }
        }
        WebViewEvent::NewWindowRequested { url, allowed } => {
            app.set_status(format!("new window {url}: allowed={allowed}").into())
        }
        WebViewEvent::DownloadRequested { url, allowed, .. } => {
            app.set_status(format!("download {url}: allowed={allowed}").into())
        }
        WebViewEvent::DownloadFinished { url, success, .. } => {
            app.set_status(format!("download finished {url}: success={success}").into())
        }
    }
}

fn is_webview_activity_event(event: &WebViewEvent) -> bool {
    let WebViewEvent::IpcMessage {
        body,
        truncated: false,
        ..
    } = event
    else {
        return false;
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };

    value.get("source").and_then(serde_json::Value::as_str) == Some("fixture")
        && value.get("kind").and_then(serde_json::Value::as_str) == Some("activity")
        && value
            .get("token")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|token| token == fixture::FIXTURE_READY_TOKEN)
}

fn regression_bounds(app: &App) -> WebViewBounds {
    let window = app.window();
    let size = window.size();
    let scale = f64::from(window.scale_factor()).max(1.0);
    let logical_width = f64::from(size.width) / scale;
    let logical_height = f64::from(size.height) / scale;

    WebViewBounds::new(
        300.0,
        64.0,
        (logical_width - 316.0).max(220.0),
        (logical_height - 104.0).max(180.0),
    )
}
