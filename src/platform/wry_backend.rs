use std::sync::mpsc::Sender;
#[cfg(target_os = "linux")]
use std::{cell::Cell, cell::RefCell, rc::Rc};

#[cfg(target_os = "linux")]
use gdkx11::X11Window;
#[cfg(target_os = "linux")]
use gtk::{glib, prelude::*};
use raw_window_handle::HasWindowHandle;
#[cfg(target_os = "linux")]
use raw_window_handle::RawWindowHandle;
#[cfg(target_os = "linux")]
use wry::WebViewExtUnix;
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::{NewWindowResponse, PageLoadEvent, Rect, WebView, WebViewBuilder};
#[cfg(target_os = "linux")]
use x11rb::CURRENT_TIME;
#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{
    ConfigureWindowAux, ConnectionExt as XprotoConnectionExt, InputFocus,
};

use crate::{
    NavigationDecision, Result, ScriptRequestId, WebViewBounds, WebViewCapabilities, WebViewError,
    WebViewEvent, WebViewOptions, WebViewSource,
};

pub struct NativeWebView {
    inner: WebView,
    event_sender: Sender<WebViewEvent>,
    #[cfg(target_os = "linux")]
    host_x11_window: Option<u32>,
    #[cfg(target_os = "linux")]
    keyboard_focus_enabled: Rc<Cell<bool>>,
    #[cfg(target_os = "linux")]
    webview_x11_roots: Rc<RefCell<Vec<u32>>>,
}

impl NativeWebView {
    pub fn attach<W>(
        window: &W,
        options: WebViewOptions,
        event_sender: Sender<WebViewEvent>,
    ) -> Result<Self>
    where
        W: HasWindowHandle,
    {
        let raw_window = window
            .window_handle()
            .map_err(|error| WebViewError::WindowHandle(error.to_string()))?
            .as_raw();
        #[cfg(target_os = "linux")]
        let host_x11_window = linux_host_x11_window(raw_window);
        #[cfg(not(target_os = "linux"))]
        let _ = raw_window;
        #[cfg(target_os = "linux")]
        let keyboard_focus_enabled = Rc::new(Cell::new(options.focused));
        #[cfg(target_os = "linux")]
        let webview_x11_roots = Rc::new(RefCell::new(Vec::new()));

        let ipc_limit = options.ipc_message_limit;
        let navigation_policy = options.navigation_policy.clone();
        let allow_popups = options.allow_popups;
        let downloads_enabled = options.downloads_enabled;

        let mut builder = WebViewBuilder::new()
            .with_bounds(to_wry_rect(options.bounds))
            .with_visible(options.visible)
            .with_focused(options.focused)
            .with_devtools(options.devtools)
            .with_clipboard(options.clipboard_enabled)
            .with_ipc_handler({
                let event_sender = event_sender.clone();
                move |request| {
                    let (body, truncated) = truncate_ipc_body(request.body(), ipc_limit);
                    let _ = event_sender.send(WebViewEvent::IpcMessage {
                        uri: request.uri().to_string(),
                        body,
                        truncated,
                    });
                }
            })
            .with_navigation_handler({
                let event_sender = event_sender.clone();
                move |url| {
                    let decision = navigation_policy.decide(&url);
                    let allow = decision == NavigationDecision::Allow;
                    let _ = event_sender.send(WebViewEvent::NavigationRequested { url, decision });
                    allow
                }
            })
            .with_on_page_load_handler({
                let event_sender = event_sender.clone();
                move |event, url| {
                    let message = match event {
                        PageLoadEvent::Started => WebViewEvent::NavigationStarted { url },
                        PageLoadEvent::Finished => WebViewEvent::NavigationFinished { url },
                    };
                    let _ = event_sender.send(message);
                }
            })
            .with_document_title_changed_handler({
                let event_sender = event_sender.clone();
                move |title| {
                    let _ = event_sender.send(WebViewEvent::TitleChanged { title });
                }
            })
            .with_download_started_handler({
                let event_sender = event_sender.clone();
                move |url, path| {
                    let _ = event_sender.send(WebViewEvent::DownloadRequested {
                        url,
                        suggested_path: Some(path.to_string_lossy().into_owned()),
                        allowed: downloads_enabled,
                    });
                    downloads_enabled
                }
            })
            .with_download_completed_handler({
                let event_sender = event_sender.clone();
                move |url, path, success| {
                    let _ = event_sender.send(WebViewEvent::DownloadFinished {
                        url,
                        path: path.map(|path| path.to_string_lossy().into_owned()),
                        success,
                    });
                }
            })
            .with_new_window_req_handler({
                let event_sender = event_sender.clone();
                move |url, _features| {
                    let _ = event_sender.send(WebViewEvent::NewWindowRequested {
                        url,
                        allowed: allow_popups,
                    });
                    if allow_popups {
                        NewWindowResponse::Allow
                    } else {
                        NewWindowResponse::Deny
                    }
                }
            });

        if !options.javascript_enabled {
            builder = builder.with_javascript_disabled();
        }

        if options.incognito {
            builder = builder.with_incognito(true);
        }

        if let Some(user_agent) = options.user_agent {
            builder = builder.with_user_agent(user_agent);
        }

        for script in options.initialization_scripts {
            builder = builder.with_initialization_script(script);
        }

        builder = match options.source {
            WebViewSource::Blank => builder.with_html(""),
            WebViewSource::Url(url) => builder.with_url(url),
            WebViewSource::Html(html) => builder.with_html(html),
        };

        let inner = builder
            .build_as_child(window)
            .map_err(|error| WebViewError::Native(error.to_string()))?;
        #[cfg(target_os = "linux")]
        install_linux_focus_gate(
            &inner,
            event_sender.clone(),
            Rc::clone(&keyboard_focus_enabled),
            Rc::clone(&webview_x11_roots),
            host_x11_window,
        );
        #[cfg(not(target_os = "linux"))]
        install_linux_focus_gate(&inner, event_sender.clone());

        Ok(Self {
            inner,
            event_sender,
            #[cfg(target_os = "linux")]
            host_x11_window,
            #[cfg(target_os = "linux")]
            keyboard_focus_enabled,
            #[cfg(target_os = "linux")]
            webview_x11_roots,
        })
    }

    pub fn capabilities() -> WebViewCapabilities {
        WebViewCapabilities::wry_native()
    }

    pub fn set_bounds(&self, bounds: WebViewBounds) -> Result<()> {
        self.inner
            .set_bounds(to_wry_rect(bounds))
            .map_err(|error| WebViewError::Native(error.to_string()))?;
        #[cfg(target_os = "linux")]
        {
            refresh_linux_webview_x11_roots(&self.inner, &self.webview_x11_roots);
            if let Some(host_window) = self.host_x11_window {
                let roots = self.webview_x11_roots.borrow();
                let _ = configure_linux_webview_x11_bounds(host_window, &roots, bounds);
            }
        }
        flush_linux_webview_layout(&self.inner, bounds);
        Ok(())
    }

    pub fn set_visible(&self, visible: bool) -> Result<()> {
        self.inner
            .set_visible(visible)
            .map_err(|error| WebViewError::Native(error.to_string()))
    }

    pub fn load_html(&self, html: &str) -> Result<()> {
        self.inner
            .load_html(html)
            .map_err(|error| WebViewError::Native(error.to_string()))
    }

    pub fn load_url(&self, url: &str) -> Result<()> {
        self.inner
            .load_url(url)
            .map_err(|error| WebViewError::Native(error.to_string()))
    }

    pub fn evaluate_script(&self, script: &str, request_id: ScriptRequestId) -> Result<()> {
        let event_sender = self.event_sender.clone();
        self.inner
            .evaluate_script_with_callback(script, move |value| {
                let _ = event_sender.send(WebViewEvent::ScriptResult { request_id, value });
            })
            .map_err(|error| WebViewError::Native(error.to_string()))
    }

    pub fn focus(&self) -> Result<()> {
        self.inner
            .focus()
            .map_err(|error| WebViewError::Native(error.to_string()))
    }

    pub fn set_keyboard_focus_enabled(&self, enabled: bool) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.keyboard_focus_enabled.set(enabled);
            refresh_linux_webview_x11_roots(&self.inner, &self.webview_x11_roots);
        }
        set_keyboard_focus_enabled(&self.inner, enabled);
        if !enabled {
            self.focus_parent()?;
            #[cfg(target_os = "linux")]
            self.restore_host_focus_if_webview_has_it();
        }
        Ok(())
    }

    pub fn focus_parent(&self) -> Result<()> {
        self.inner
            .focus_parent()
            .map_err(|error| WebViewError::Native(error.to_string()))?;
        self.focus_host_window()
    }
}

pub fn initialize_platform() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        gtk::init().map_err(|error| WebViewError::Platform(error.to_string()))?;
    }

    Ok(())
}

pub fn pump_platform_events() {
    #[cfg(target_os = "linux")]
    {
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }
}

fn to_wry_rect(bounds: WebViewBounds) -> Rect {
    Rect {
        position: LogicalPosition::new(bounds.x, bounds.y).into(),
        size: LogicalSize::new(bounds.width, bounds.height).into(),
    }
}

#[cfg(target_os = "linux")]
fn linux_host_x11_window(raw_window: RawWindowHandle) -> Option<u32> {
    match raw_window {
        RawWindowHandle::Xlib(handle) => u32::try_from(handle.window)
            .ok()
            .filter(|window| *window != 0),
        RawWindowHandle::Xcb(handle) => Some(handle.window.get()),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
impl NativeWebView {
    fn focus_host_window(&self) -> Result<()> {
        let Some(host_window) = self.host_x11_window else {
            return Ok(());
        };

        focus_x11_window(host_window)
    }

    fn restore_host_focus_if_webview_has_it(&self) {
        let Some(host_window) = self.host_x11_window else {
            return;
        };
        let roots = self.webview_x11_roots.borrow();
        let _ = restore_x11_focus_if_descendant(host_window, &roots);
    }
}

#[cfg(not(target_os = "linux"))]
impl NativeWebView {
    fn focus_host_window(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn install_linux_focus_gate(
    webview: &WebView,
    event_sender: Sender<WebViewEvent>,
    keyboard_focus_enabled: Rc<Cell<bool>>,
    webview_x11_roots: Rc<RefCell<Vec<u32>>>,
    host_x11_window: Option<u32>,
) {
    let widget = webview.webview();
    refresh_linux_webview_x11_roots(webview, &webview_x11_roots);
    set_widget_keyboard_focus(&widget, keyboard_focus_enabled.get());
    widget.add_events(
        gtk::gdk::EventMask::ENTER_NOTIFY_MASK
            | gtk::gdk::EventMask::LEAVE_NOTIFY_MASK
            | gtk::gdk::EventMask::BUTTON_PRESS_MASK
            | gtk::gdk::EventMask::KEY_PRESS_MASK
            | gtk::gdk::EventMask::POINTER_MOTION_MASK,
    );

    widget.connect_enter_notify_event({
        let event_sender = event_sender.clone();
        let keyboard_focus_enabled = Rc::clone(&keyboard_focus_enabled);
        let webview_x11_roots = Rc::clone(&webview_x11_roots);
        move |widget, _event| {
            if !keyboard_focus_enabled.get() {
                deny_webview_keyboard_focus(widget, host_x11_window, &webview_x11_roots);
                let _ = event_sender.send(WebViewEvent::FocusChanged { focused: false });
            }
            glib::Propagation::Proceed
        }
    });
    widget.connect_focus_in_event({
        let event_sender = event_sender.clone();
        let keyboard_focus_enabled = Rc::clone(&keyboard_focus_enabled);
        let webview_x11_roots = Rc::clone(&webview_x11_roots);
        move |widget, _event| {
            if keyboard_focus_enabled.get() {
                let _ = event_sender.send(WebViewEvent::FocusChanged { focused: true });
            } else {
                deny_webview_keyboard_focus(widget, host_x11_window, &webview_x11_roots);
                let _ = event_sender.send(WebViewEvent::FocusChanged { focused: false });
            }
            glib::Propagation::Proceed
        }
    });
    widget.connect_focus_out_event({
        let event_sender = event_sender.clone();
        let keyboard_focus_enabled = Rc::clone(&keyboard_focus_enabled);
        move |_widget, _event| {
            keyboard_focus_enabled.set(false);
            let _ = event_sender.send(WebViewEvent::FocusChanged { focused: false });
            glib::Propagation::Proceed
        }
    });
    widget.connect_key_press_event({
        let event_sender = event_sender.clone();
        let keyboard_focus_enabled = Rc::clone(&keyboard_focus_enabled);
        let webview_x11_roots = Rc::clone(&webview_x11_roots);
        move |widget, _event| {
            if keyboard_focus_enabled.get() {
                glib::Propagation::Proceed
            } else {
                deny_webview_keyboard_focus(widget, host_x11_window, &webview_x11_roots);
                let _ = event_sender.send(WebViewEvent::FocusChanged { focused: false });
                glib::Propagation::Stop
            }
        }
    });
    widget.connect_button_press_event(move |widget, _event| {
        keyboard_focus_enabled.set(true);
        set_widget_keyboard_focus(widget, true);
        let _ = event_sender.send(WebViewEvent::FocusRequested);
        glib::Propagation::Proceed
    });
}

#[cfg(not(target_os = "linux"))]
fn install_linux_focus_gate(_webview: &WebView, _event_sender: Sender<WebViewEvent>) {}

#[cfg(target_os = "linux")]
fn set_keyboard_focus_enabled(webview: &WebView, enabled: bool) {
    let widget = webview.webview();
    set_widget_keyboard_focus(&widget, enabled);
}

#[cfg(not(target_os = "linux"))]
fn set_keyboard_focus_enabled(_webview: &WebView, _enabled: bool) {}

#[cfg(target_os = "linux")]
fn flush_linux_webview_layout(webview: &WebView, _bounds: WebViewBounds) {
    let widget = webview.webview();
    widget.queue_resize();
    widget.queue_draw();
    if let Some(window) = widget.window() {
        window.invalidate_rect(None, true);
    }
}

#[cfg(not(target_os = "linux"))]
fn flush_linux_webview_layout(_webview: &WebView, _bounds: WebViewBounds) {}

#[cfg(target_os = "linux")]
fn set_widget_keyboard_focus<W: gtk::prelude::WidgetExt>(widget: &W, enabled: bool) {
    widget.set_can_focus(enabled);
    widget.set_focus_on_click(enabled);
}

#[cfg(target_os = "linux")]
fn deny_webview_keyboard_focus<W: gtk::prelude::WidgetExt>(
    widget: &W,
    host_x11_window: Option<u32>,
    webview_x11_roots: &RefCell<Vec<u32>>,
) {
    set_widget_keyboard_focus(widget, false);
    if let Some(window) = widget
        .toplevel()
        .and_then(|toplevel| toplevel.downcast::<gtk::Window>().ok())
    {
        window.set_focus(Option::<&gtk::Widget>::None);
    }
    if let Some(host_window) = host_x11_window {
        let roots = webview_x11_roots.borrow();
        let _ = restore_x11_focus_if_descendant(host_window, &roots);
        let _ = focus_x11_window(host_window);
    }
}

#[cfg(target_os = "linux")]
fn refresh_linux_webview_x11_roots(webview: &WebView, roots: &RefCell<Vec<u32>>) {
    let widget = webview.webview();
    let mut next = Vec::new();

    if let Some(window) = widget.window() {
        push_x11_window_id(&mut next, &window);
    }

    if let Some(window) = widget.toplevel().and_then(|toplevel| toplevel.window()) {
        push_x11_window_id(&mut next, &window);
    }

    next.sort_unstable();
    next.dedup();
    *roots.borrow_mut() = next;
}

#[cfg(target_os = "linux")]
fn push_x11_window_id(windows: &mut Vec<u32>, window: &gtk::gdk::Window) {
    if let Ok(window) = window.clone().downcast::<X11Window>() {
        let xid = window.xid();
        if let Ok(xid) = u32::try_from(xid)
            && xid != 0
        {
            windows.push(xid);
        }
    }
}

#[cfg(target_os = "linux")]
fn restore_x11_focus_if_descendant(host_window: u32, roots: &[u32]) -> Result<()> {
    if roots.is_empty() {
        return focus_x11_window(host_window);
    }

    let (connection, _) =
        x11rb::connect(None).map_err(|error| WebViewError::Native(error.to_string()))?;
    let focused_window = connection
        .get_input_focus()
        .map_err(|error| WebViewError::Native(error.to_string()))?
        .reply()
        .map_err(|error| WebViewError::Native(error.to_string()))?
        .focus;

    if focused_window == host_window || !is_x11_descendant(&connection, focused_window, roots) {
        return Ok(());
    }

    connection
        .set_input_focus(InputFocus::PARENT, host_window, CURRENT_TIME)
        .map_err(|error| WebViewError::Native(error.to_string()))?;
    connection
        .flush()
        .map_err(|error| WebViewError::Native(error.to_string()))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn configure_linux_webview_x11_bounds(
    host_window: u32,
    roots: &[u32],
    bounds: WebViewBounds,
) -> Result<()> {
    let Some(container) = linux_webview_container_window(host_window, roots)? else {
        return Ok(());
    };

    let x = bounds.x.round() as i32;
    let y = bounds.y.round() as i32;
    let width = bounds.width.round().max(1.0) as u32;
    let height = bounds.height.round().max(1.0) as u32;

    let (connection, _) =
        x11rb::connect(None).map_err(|error| WebViewError::Native(error.to_string()))?;
    let container_bounds = ConfigureWindowAux {
        x: Some(x),
        y: Some(y),
        width: Some(width),
        height: Some(height),
        border_width: None,
        sibling: None,
        stack_mode: None,
    };
    connection
        .configure_window(container, &container_bounds)
        .map_err(|error| WebViewError::Native(error.to_string()))?;

    if let Ok(cookie) = connection.query_tree(container)
        && let Ok(reply) = cookie.reply()
    {
        let child_bounds = ConfigureWindowAux {
            x: Some(0),
            y: Some(0),
            width: Some(width),
            height: Some(height),
            border_width: None,
            sibling: None,
            stack_mode: None,
        };
        for child in reply.children {
            let _ = connection.configure_window(child, &child_bounds);
        }
    }

    connection
        .flush()
        .map_err(|error| WebViewError::Native(error.to_string()))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_webview_container_window(host_window: u32, roots: &[u32]) -> Result<Option<u32>> {
    let (connection, _) =
        x11rb::connect(None).map_err(|error| WebViewError::Native(error.to_string()))?;

    for root in roots {
        let mut candidate = *root;
        for _ in 0..32 {
            if candidate == host_window {
                break;
            }

            let Ok(cookie) = connection.query_tree(candidate) else {
                break;
            };
            let Ok(reply) = cookie.reply() else {
                break;
            };
            if reply.parent == host_window {
                return Ok(Some(candidate));
            }
            if reply.parent == 0 || reply.parent == candidate {
                break;
            }
            candidate = reply.parent;
        }
    }

    let Ok(cookie) = connection.query_tree(host_window) else {
        return Ok(None);
    };
    let Ok(reply) = cookie.reply() else {
        return Ok(None);
    };

    for child in &reply.children {
        if roots.contains(child)
            || roots
                .iter()
                .any(|root| x11_descendants(&connection, *child).contains(root))
        {
            return Ok(Some(*child));
        }
    }

    Ok(reply.children.into_iter().next())
}

#[cfg(target_os = "linux")]
fn is_x11_descendant<C: Connection>(connection: &C, window: u32, roots: &[u32]) -> bool {
    if roots.contains(&window) {
        return true;
    }

    for root in roots {
        if x11_descendants(connection, *root).contains(&window) {
            return true;
        }
    }

    false
}

#[cfg(target_os = "linux")]
fn x11_descendants<C: Connection>(connection: &C, root: u32) -> Vec<u32> {
    let mut descendants = Vec::new();
    let mut pending = vec![root];

    while let Some(window) = pending.pop() {
        let Ok(cookie) = connection.query_tree(window) else {
            continue;
        };
        let Ok(reply) = cookie.reply() else {
            continue;
        };
        for child in reply.children {
            descendants.push(child);
            pending.push(child);
        }
    }

    descendants
}

#[cfg(target_os = "linux")]
fn focus_x11_window(window: u32) -> Result<()> {
    let (connection, _) =
        x11rb::connect(None).map_err(|error| WebViewError::Native(error.to_string()))?;
    connection
        .set_input_focus(InputFocus::PARENT, window, CURRENT_TIME)
        .map_err(|error| WebViewError::Native(error.to_string()))?;
    connection
        .flush()
        .map_err(|error| WebViewError::Native(error.to_string()))?;
    Ok(())
}

fn truncate_ipc_body(body: &str, limit: usize) -> (String, bool) {
    if body.len() <= limit {
        return (body.to_owned(), false);
    }

    let mut boundary = limit;
    while boundary > 0 && !body.is_char_boundary(boundary) {
        boundary -= 1;
    }

    (body[..boundary].to_owned(), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_truncation_preserves_utf8_boundaries() {
        let (body, truncated) = truncate_ipc_body("aéz", 2);

        assert_eq!(body, "a");
        assert!(truncated);
    }
}
