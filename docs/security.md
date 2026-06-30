# Security Model

The crate embeds platform webviews, so web content should be treated as active
untrusted code unless the application controls every byte loaded into it.

## Defaults

`WebViewOptions::default()` is intentionally conservative:

- Blank source
- JavaScript disabled
- Devtools disabled
- Clipboard disabled
- Downloads disabled
- Popups disabled
- Initial webview focus disabled
- Incognito disabled
- 1 MiB IPC message limit

Applications must opt in to each browser capability they need.

## Remote Content

For remote URLs, prefer an allow-list navigation policy. Treat IPC messages as
untrusted input, even if they come from a trusted origin, because navigation can
change the loaded document.

## Local HTML

Inline HTML is useful for deterministic tools and docs, but enabling JavaScript
for local HTML still runs active code inside the webview. Keep initialization
scripts minimal and version-controlled.

## IPC

IPC payloads are bounded and marked as truncated when the configured byte limit
is exceeded. Applications should parse structured payloads with a real parser
and reject unexpected fields.

## Downloads

Downloads are disabled by default. When enabled, the backend reports download
request and completion events. Applications remain responsible for deciding
where downloaded files belong and whether they should be opened.

## Devtools

Devtools are disabled by default and should stay off in normal production
builds. On Windows, WebView2 remote debugging can also be enabled externally
through `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`; only use that in trusted test
environments.
