# Product Requirements: Slint Webview Composition

## Goal

Provide a small native-webview integration for Slint applications that can host
web content next to normal Slint controls without embedding a full browser
engine in the application binary.

The first production-shaped milestone is a stable controller API, predictable
native-child composition behavior, deterministic regression coverage, and clear
documentation. Publishing is explicitly out of scope for this milestone.

## Users

- Slint app developers who need to show documentation, authentication pages,
  local HTML tools, dashboards, or small embedded web experiences.
- Maintainers evaluating whether native platform webviews can become a reusable
  Slint composition layer.
- Test authors validating that normal Slint UI still works while a native
  webview is attached.

## Requirements

- Use native platform webviews through Wry for the first backend.
- Keep Wry and backend internals out of the public API.
- Expose `WebViewController` as the primary application-facing type.
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
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `cargo test --no-default-features` passes.
- `cargo test` passes.
- `cargo doc --all-features --no-deps` passes.
- Regression app compiles behind `--features testing`.
- Windows smoke proves attach, DOM probe, event loop operation, and clean exit.
- Windows visual smoke captures the composed application and validates sentinel
  colors.
- WSL/Linux smoke proves attach and DOM probe when native dependencies exist.
- README, API docs, architecture docs, security docs, limitations, and platform
  notes are present.

## Open Product Decisions

- Whether a future release should expose a Slint component wrapper in addition
  to `WebViewController`.
- Whether macOS is a first verified platform before publish.
- Whether custom protocols should be exposed in the first public API.
- Whether async/future-based script evaluation should wrap the current event
  correlation model.
