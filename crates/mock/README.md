# slint-webview-mock

`slint-webview-mock` is a deterministic rendered backend for tests and examples.
It implements the same shared browser and rendered-surface contracts as the
planned Servo and CEF backends, but it does not embed a browser engine.

Use it when you need to verify Slint-side composition, input routing, frame
draining, and controller behavior without depending on platform webviews or a
large rendered engine runtime.
