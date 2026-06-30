# Testing Strategy

The test strategy has three layers.

## API Tests

Run:

```powershell
cargo test --no-default-features
cargo test
```

These verify serializable API types, conservative defaults, navigation policy,
and pure Rust behavior. The no-default-features path proves the public API can
compile without a native backend.

## Static Gates

Run:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --all-features --no-deps
```

Warnings are treated as failures for production-shape work.

## Native Regression App

The regression app is compiled behind `--features testing`:

```powershell
cargo run --features testing --bin slint-webview-regression -- --smoke
```

The smoke path attaches a native webview, loads a deterministic fixture, probes
the DOM, and exits when the fixture reports readiness.

The Windows visual smoke captures the composed app window and validates sentinel
colors from both Slint UI and the webview fixture:

```powershell
.\scripts\check.ps1 -Smoke -Visual
```

The fixture is local and deterministic by design. External sites such as YouTube
are useful for manual stress testing, but they are not stable regression inputs.

## Package Validation

Run before cutting a release candidate:

```powershell
.\scripts\package.ps1
```

or:

```bash
bash scripts/package.sh
```

These scripts run formatting, clippy, API tests with and without the native
backend, rustdoc, and `cargo package --allow-dirty`. The Linux script exits with
code 20 when native GTK/WebKitGTK packages are missing after proving the
no-default-features API path.
