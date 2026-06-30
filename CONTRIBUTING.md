# Contributing

`slint-webview` is a native-child webview controller for Slint applications.
Changes should preserve the public controller API and keep backend-specific
details private unless there is a clear reason to expose them.

## Development Checks

Run the platform check before submitting changes:

```powershell
.\scripts\check.ps1
```

On Linux or WSL:

```bash
bash scripts/check.sh
```

For release packaging:

```powershell
.\scripts\package.ps1
```

or:

```bash
bash scripts/package.sh
```

## Design Constraints

- Keep default options conservative.
- Keep native backend dependencies behind feature flags where possible.
- Treat the webview as a native child surface, not a Slint-rendered item.
- Document platform differences instead of hiding them.
- Avoid adding `unsafe` code to this crate.
- Keep regression inputs deterministic; use local fixtures over external sites.

## Platform Testing

Windows uses WebView2. Linux uses WebKitGTK through Wry. WSLg is useful for
regression work but does not replace native Linux validation. macOS support
must be checked on macOS before claiming release coverage.
