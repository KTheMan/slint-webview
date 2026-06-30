# Product Requirements: Slint Webview Composition

## Goal

Provide a small native-webview integration for Slint applications that can host
web content next to normal Slint controls without embedding a full browser
engine in the application binary.

The first production-shaped milestone is a stable controller API, a reusable
Slint-facing area wrapper for native-child composition, deterministic regression
coverage, and clear documentation. Publishing is explicitly out of scope for
this milestone.

The follow-on product direction is now represented in the workspace by a shared
core plus separate native, Servo, and CEF backend crates. Native targets minimum
footprint and platform fidelity; Servo and CEF target more uniform Slint-owned
composition.

## Users

- Slint app developers who need to show documentation, authentication pages,
  local HTML tools, dashboards, or small embedded web experiences.
- Maintainers evaluating whether native platform webviews can become a reusable
  Slint composition layer.
- Maintainers evaluating Servo or CEF as consistent texture-rendered backends
  behind the same Slint-facing API.
- Test authors validating that normal Slint UI still works while a native
  webview is attached.

## Requirements

- Use native platform webviews through Wry for the first backend.
- Keep Wry and backend internals out of the public API.
- Expose `WebViewController` as the primary low-level application-facing type.
- Expose `WebViewAreaController` and `ui/webview-area.slint` for apps that want
  widget-style Slint composition policy.
- Support loading blank, URL, and inline HTML sources.
- Support logical-pixel bounds updates from Slint layout code.
- Support show/hide, focus, parent-focus restore, JavaScript evaluation, IPC,
  title change events, navigation events, popup policy, and download policy.
- Keep risky browser capabilities disabled by default.
- Keep deterministic test fixtures behind the `testing` feature.
- Provide a regression app that exercises webview composition beside Slint
  controls.
- Provide automated programmatic and visual verification on Windows, plus a
  WSL/Linux smoke path.

## Non-Goals

- Rendering web content into a Slint-owned texture.
- Solving arbitrary clipping, opacity, transform, or overlay behavior for native
  child views.
- Shipping a Servo-based backend.
- Publishing to crates.io in this milestone.
- Building a full browser shell.

## Acceptance Criteria

- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo test --workspace --no-default-features` passes.
- `cargo test --workspace` passes.
- `cargo doc --workspace --all-features --no-deps` passes.
- Regression app compiles behind `--features testing`.
- Windows smoke proves attach, DOM probe, event loop operation, and clean exit.
- Windows visual smoke captures the composed application and validates sentinel
  colors.
- WSL/Linux smoke proves attach and DOM probe when native dependencies exist.
- README, API docs, architecture docs, WebViewArea docs, security docs,
  backend strategy docs, limitations, and platform notes are present.

## Open Product Decisions

- Whether the Slint component wrapper should use Slint's experimental crate
  module import path before publishing.
- Whether macOS is a first verified platform before publish.
- Whether custom protocols should be exposed in the first public API.
- Whether async/future-based script evaluation should wrap the current event
  correlation model.
- Whether Servo or CEF should be the first non-native backend pursued after the
  native crate split.
