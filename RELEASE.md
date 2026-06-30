# Release Checklist

This crate is release-shaped but `publish = false` remains set until the crate
owner chooses the crates.io publishing policy.

## Required Before Publishing

- Decide whether the public crate name remains `slint-webview`.
- Confirm `repository` and `homepage` metadata still point to the intended
  public repository.
- Decide whether `Cargo.lock` should remain in the package.
- Confirm the public stability label for `0.1.0`.
- Re-run Windows and Linux/WSL validation on clean machines.
- Verify macOS with a WKWebView smoke run or document macOS as unverified in
  the release announcement.
- Confirm LGPL-3.0-only is still the intended license.
- Decide whether `ui/webview-area.slint` should remain a source import or move
  to Slint's experimental crate module build flow.
- Decide whether the first public release should publish `slint-webview-core`
  and `slint-webview-native` as separate packages before the facade.

## Local Release Validation

Windows:

```powershell
.\scripts\package.ps1
.\scripts\check.ps1 -Smoke -Visual
```

Linux or WSL:

```bash
bash scripts/package.sh
bash scripts/check.sh --smoke
```

## Package Dry Run

The package scripts run:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --no-default-features`
- `cargo test --workspace`
- `cargo doc --workspace --all-features --no-deps`
- `cargo package -p slint-webview-core --allow-dirty`

Use `cargo package -p slint-webview-core --list` to inspect the core tarball
contents. The facade crate cannot be package-verified until path dependencies
such as `slint-webview-core` and `slint-webview-native` are published to the
selected registry or vendored for the release, because Cargo strips path
dependencies during package preparation.
The manifests have explicit `include` lists so build output and local WebView2
profile state cannot enter release packages.

## Publishing

Publishing is intentionally blocked:

```toml
publish = false
```

Remove that line only as part of a deliberate release commit after the required
publishing and platform decisions are complete.
