# Production Readiness Audit

Status: Release-candidate shaped, publishing intentionally blocked
Date: 2026-06-29
Scope: `slint-webview` crate functionality, API shape, docs, testing, and package hygiene before a first public crate release.

This audit records whether the crate is production-shaped enough to package and
review as a first release candidate. The publishing decision is tracked
separately in `RELEASE.md`.

## Current Judgment

The workspace is production-shaped enough for internal integration, core
package dry-run, and release-candidate review. It should still remain
`publish = false` until public stability, repository metadata, Cargo.lock
policy, package ordering, and macOS verification decisions are made.

The most important hardening work is complete:

- Public API is centered on core re-exports, `WebViewController`, and the Tier
  1.5 `WebViewAreaController`; backend types are private.
- Defaults are conservative and no longer load fixture content.
- Fixture helpers and the regression app are behind the `testing` feature.
- Navigation policy is caller-configurable.
- Script evaluation has request IDs.
- IPC messages include URI and truncation status, with byte-safe truncation.
- Downloads, popups, clipboard, devtools, JavaScript, initial webview focus, and
  incognito mode are explicit opt-ins.
- Bounds are documented and validated before attach/update.
- Rustdoc is enabled with `#![warn(missing_docs)]`.
- README and standalone docs cover PRD, spec, architecture, API, WebViewArea
  composition, backend strategy, security, testing, limitations, platform
  notes, and licensing.
- Canonical check scripts run formatting, clippy, tests, docs, and regression
  binary checks.
- Release package scripts run the static gates and `cargo package` for
  `slint-webview-core`.
- The manifest has an explicit package `include` list.
- Shared API, area-policy types, and area-controller behavior now live in
  `slint-webview-core`.
- Shared backend-controller behavior now lives in `BackendWebViewController`.
- Shared Slint placeholder synchronization, parking, overlay, and focus policy
  now live in `BackendWebViewAreaController`.
- Shared rendered-frame and Slint-originated input contracts now live in
  `RenderedWebViewBackend` and related core types for future Servo/CEF work.
- The native Wry backend lives in `slint-webview-native`; Servo and CEF backend
  crate shells compile against the shared core.
- Linux native backend dependencies are optional behind `backend-wry`, so
  `--no-default-features` remains a pure API build.

## Required Gates

Run on Windows before considering a change ready:

```powershell
.\scripts\package.ps1
.\scripts\check.ps1
.\scripts\check.ps1 -Smoke -Visual
```

Run for WSL/Linux validation:

```powershell
.\scripts\test-wsl.ps1
.\scripts\test-wsl.ps1 -Smoke
```

`scripts/check.sh` also works directly from Linux shells.

## API Review

The primary API is:

```rust
let controller = WebViewController::attach(window_handle, WebViewOptions::default())?;
controller.set_bounds(bounds)?;
let request_id = controller.evaluate_script("document.title")?;
for event in controller.drain_events() {
    // handle event
}
```

This is the right low-level shape for the current native-child composition tier.
The crate now also exposes `WebViewAreaController` and `ui/webview-area.slint`
for reusable Slint-facing parking, overlay, and focus policy without exposing
Wry.

Current API caveats:

- `WebViewOptions` is a public-field struct. That is convenient now, but a
  builder-only or `#[non_exhaustive]` API should be reconsidered before a stable
  public release.
- Script results are event-based, not `Future`-based.
- `NavigationDecision::OpenExternal` is represented but not automatically
  executed by the crate.
- Runtime diagnostics are still mostly attach-time errors rather than a rich
  preflight report.
- The shipped Slint component is a source component import, not yet a Slint
  experimental crate module.
- The repository has been mechanically split enough for the facade to depend on
  `slint-webview-native`; Servo and CEF remain crate shells.

## Functional Review

Implemented:

- Attach native webview to an existing native window handle.
- Load blank, URL, and inline HTML sources.
- Resize and show/hide.
- Focus and parent-focus restore where supported.
- Configurable JavaScript, devtools, clipboard, initial focus, popups,
  downloads, incognito, user agent, initialization scripts, IPC limit, and
  navigation policy.
- Structured events for navigation, IPC, title, script result, popup, and
  download activity.
- Static capability reporting for the selected backend.
- `WebViewAreaController` parking, overlay suppression, focus release, and
  event-policy handling.

Not yet implemented:

- True Slint scene-graph widget.
- Slint-owned texture rendering.
- Public custom protocol API.
- Permission interception.
- Verified macOS smoke coverage in this workspace.
- Rich runtime diagnostics for missing platform webview runtimes.

## Documentation Review

Documentation now exists in:

- `README.md`
- `docs/prd.md`
- `docs/spec.md`
- `docs/architecture.md`
- `docs/api.md`
- `docs/security.md`
- `docs/testing.md`
- `docs/webview-area.md`
- `docs/backend-crate-strategy.md`
- `docs/limitations.md`
- `docs/platforms/windows.md`
- `docs/platforms/linux-wsl.md`
- `docs/platforms/macos.md`
- `docs/licensing.md`
- `CHANGELOG.md`
- `RELEASE.md`
- `CONTRIBUTING.md`
- `SECURITY.md`

The docs intentionally state that the current implementation is a native child
view, not a true Slint scene-graph item.

## Testing Review

Current regression coverage proves:

- Pure API behavior with and without the default backend.
- Pure `WebViewAreaPolicy` placement behavior with and without the default
  backend.
- Pure `WebViewBackend` shared controller behavior.
- Pure `BackendWebViewAreaController` sync, parking, event-pump, and
  focus-policy behavior.
- Pure rendered-backend frame, transport, resize, and input contract behavior.
- Workspace compilation for core plus native, Servo, and CEF crates.
- Formatting, clippy, and rustdoc gates.
- `examples/area.rs` compilation through all-target checks.
- Regression app compilation behind `testing`.
- Windows WebView2 smoke attach and DOM probe.
- Windows composed-window visual sentinel check.
- WSL WebKitGTK smoke attach and DOM probe when dependencies are installed.
- `slint-webview-core` package dry-run on Windows.

Coverage that should be added before public release:

- Automated focus recovery checks.
- Automated hide/show and modal policy checks.
- IPC round-trip assertions.
- Download allowed/blocked assertions.
- Popup blocked assertions.
- Navigation policy blocked/allowed assertions inside the native regression app.
- macOS smoke run.

## Package Hygiene

Current package state:

- `publish = false` remains set.
- License is `LGPL-3.0-only`.
- The manifest declares README, documentation, keywords, categories, and an
  explicit package include list.
- `.gitignore` excludes local build and editor artifacts.
- Package metadata points at the public GitHub repository while crates.io
  publishing remains intentionally blocked.

Before publish, decide:

- Public crate name.
- Whether `slint-webview-core` and `slint-webview-native` are published first or
  vendored before the facade package is dry-run verified.
- Whether to include `Cargo.lock`.
- Public stability label: alpha, preview, or experimental.

## Recommendation

Use this crate as a release candidate for internal integration and continued
platform validation. Do not publish yet. The next useful work is validating
macOS, deciding public repository metadata, and expanding automated native
regression coverage around focus, modal hide/show, IPC, downloads, popups, and
navigation policy.
