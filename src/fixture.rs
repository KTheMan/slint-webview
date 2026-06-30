//! Deterministic fixture helpers for regression tests.

/// Token present in the fixture document title and DOM state.
pub const FIXTURE_READY_TOKEN: &str = "slint-webview-fixture-ready";

/// Returns the deterministic fixture HTML used by smoke and regression tests.
pub fn fixture_html() -> &'static str {
    include_str!("../fixtures/webview-fixture.html")
}

/// Returns JavaScript that serializes fixture state as JSON.
pub fn fixture_state_script() -> &'static str {
    "JSON.stringify(window.__slintWebViewTest.state())"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_contains_ready_token_and_probe_api() {
        let html = fixture_html();

        assert!(html.contains(FIXTURE_READY_TOKEN));
        assert!(html.contains("__slintWebViewTest"));
    }
}
