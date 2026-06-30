# Rendered Backend Roadmap

The rendered backend track exists to turn the webview into Slint-composited
content instead of a native child window. That is the route to normal Slint
z-order, clipping, modals, and monitor behavior.

## Current Baseline

`slint-webview-core` owns the shared contracts:

- `RenderedWebViewBackend` for surface resize, input routing, and frame output.
- `BackendRenderedWebViewController` for shared validation and event draining.
- CPU pixel and external texture frame payloads.
- Pointer, wheel, keyboard, IME, and focus input events.

`slint-webview-mock` is the first concrete rendered backend. It produces
deterministic CPU frames and records input so the Slint-side fixture can be
tested before a heavy engine is integrated.

## Engine Order

Use CEF first when the priority is real-world web compatibility. It gives a
Chromium engine, broad site coverage, and a mature offscreen rendering API at
the cost of binary size, subprocess packaging, and security update cadence.

Use Servo first when the priority is Rust-native architecture and deeper Slint
integration. It is a better conceptual fit for Slint-owned texture composition,
but it is the riskier compatibility path for arbitrary modern websites.

The recommended production path is CEF first, then Servo as the cleaner
long-term renderer path unless project priorities explicitly flip that order.

## Facade Features

The facade exposes backend-family features:

| Feature | Role |
| --- | --- |
| `backend-native` | Default native backend family |
| `backend-wry` | Wry implementation used by `backend-native` |
| `backend-cef` | Opt-in CEF rendered backend shell |
| `backend-servo` | Opt-in Servo rendered backend shell |

The mock rendered backend is a dev/test crate today. It should stay available
for examples and CI without becoming the default runtime backend.

## CEF MVP

CEF should start with windowless/offscreen rendering and CPU paint buffers.
That is the least clever path and the fastest way to prove real web
compatibility inside Slint composition.

Package layout:

- `slint-webview-cef` owns the CEF adapter and implements `WebViewBackend` plus
  `RenderedWebViewBackend`.
- A separate helper binary owns the CEF subprocess entry point. Applications
  must ship it beside the main executable.
- CEF framework binaries, resource packs, locales, and snapshot data are copied
  into an engine runtime directory selected by the package script.
- The main process discovers the runtime directory through an explicit option
  or environment variable before falling back to executable-relative paths.
- Debug builds may allow a developer-supplied CEF binary path; release builds
  should be deterministic.

CPU paint-buffer MVP:

1. Initialize CEF once per process and register the subprocess path.
2. Create a windowless browser for each webview instance.
3. Translate `WebViewOptions` into CEF settings, navigation policy, JavaScript,
   popup, download, clipboard, and permission behavior.
4. On Slint resize, call CEF's windowless resize path and update the browser
   view rect in physical pixels.
5. In `OnPaint`, copy BGRA premultiplied pixels into
   `RenderedWebViewFramePayload::CpuPixels`.
6. Queue dirty rectangles from CEF into `RenderedWebViewFrame::dirty_rects`.
7. Drain frames through `BackendRenderedWebViewController::drain_frames`.
8. Translate Slint pointer, wheel, keyboard, IME, and focus events into CEF
   host input events.
9. Emit shared `WebViewEvent` values for load, title, script, focus,
   navigation, popup, download, and IPC events.

Do not begin with GPU acceleration. Add shared texture paths after CPU paint is
visually correct, focus-correct, and regression-tested.

## Servo MVP

Servo should start from the same rendered contract, but its architecture should
prefer direct texture composition when Slint can consume the output.

Event loop and ownership:

- One backend runtime owns Servo initialization and browser event pumping.
- Each webview owns a Servo browsing context plus a rendered surface size.
- Slint remains the UI event source. Pointer, wheel, keyboard, IME, clipboard,
  and focus changes are translated into Servo input events.
- The backend queues frames without blocking the Slint event loop.
- Shutdown must tear down browsing contexts before dropping shared GPU or CPU
  surfaces.

Pixel/texture MVP:

1. Implement CPU pixel output first if texture interop is not immediately
   stable on all development platforms.
2. Prefer `RenderedWebViewFramePayload::ExternalTexture` once Servo and Slint
   can safely share a GPU handle on a target backend.
3. Preserve CPU pixel fallback for diagnostics and headless-ish tests.
4. Track device scale and physical size exactly; all Slint logical input must
   be converted through the current scale factor.
5. Emit dirty rects or full-frame updates consistently.

Compatibility should be reported clearly. Servo can be the better Slint-native
architecture while still being less compatible with arbitrary websites than CEF.

## Regression Expectations

Every rendered backend should pass the same observable checks:

- The Slint shell remains interactive while the webview is visible.
- Slint overlays appear above web content without parking hacks.
- Hide/show does not leave stale native child-window artifacts.
- Resizing and monitor moves update the rendered surface without snapping back
  to stale dimensions.
- Clicking a webview text field gives the webview keyboard focus.
- Clicking a Slint text field gives the Slint shell keyboard focus.
- Keyboard input follows the currently focused element, not cursor hover.
- Script evaluation, IPC, navigation policy, popup policy, and download policy
  match shared core behavior.
- Visual sentinel checks can see Slint UI and rendered web content in one
  composed frame.
