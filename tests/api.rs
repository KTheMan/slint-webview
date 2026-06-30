use slint_webview::{
    CompositionTier, DEFAULT_PARK_BOUNDS, HiddenWebViewStrategy, NavigationDecision,
    NavigationPolicy, WebViewAreaPolicy, WebViewAreaState, WebViewBounds, WebViewCapabilities,
    WebViewOptions, WebViewSource,
};

#[test]
fn capabilities_describe_native_child_view_contract() {
    let caps = WebViewCapabilities::wry_native();

    assert_eq!(caps.composition_tier, CompositionTier::NativeChildView);
    assert!(caps.supports_host_messaging);
    assert!(!caps.supports_overlays_above_webview);
}

#[test]
fn bounds_can_be_serialized_for_diagnostics() {
    let bounds = WebViewBounds::new(1.0, 2.0, 3.0, 4.0);
    let json = serde_json::to_string(&bounds).unwrap();

    assert!(json.contains("\"width\":3.0"));
}

#[test]
fn default_options_do_not_load_fixture_or_enable_privileged_features() {
    let options = WebViewOptions::default();

    assert!(matches!(options.source, WebViewSource::Blank));
    assert!(!options.javascript_enabled);
    assert!(!options.devtools);
    assert!(!options.focused);
    assert!(!options.clipboard_enabled);
    assert!(!options.downloads_enabled);
    assert!(!options.incognito);
    assert!(options.user_agent.is_none());
    assert!(options.initialization_scripts.is_empty());
}

#[test]
fn navigation_policy_allows_scheme_allowlist() {
    let policy = NavigationPolicy::AllowSchemes(vec!["https".to_owned()]);

    assert_eq!(
        policy.decide("https://example.com"),
        NavigationDecision::Allow
    );
    assert_eq!(
        policy.decide("file:///tmp/test.html"),
        NavigationDecision::Block
    );
}

#[test]
fn area_policy_parks_hidden_webview_by_default() {
    let state = WebViewAreaState::new(WebViewBounds::new(12.0, 24.0, 320.0, 240.0))
        .with_requested_visible(false);
    let placement = WebViewAreaPolicy::default().resolve(state);

    assert!(!placement.effective_visible);
    assert!(placement.native_visible);
    assert!(placement.parked);
    assert_eq!(placement.bounds, DEFAULT_PARK_BOUNDS);
}

#[test]
fn area_policy_can_request_native_hide_instead_of_parking() {
    let policy = WebViewAreaPolicy::default().with_hidden_strategy(HiddenWebViewStrategy::Hide);
    let state = WebViewAreaState::new(WebViewBounds::new(12.0, 24.0, 320.0, 240.0))
        .with_requested_visible(false);
    let placement = policy.resolve(state);

    assert!(!placement.effective_visible);
    assert!(!placement.native_visible);
    assert!(!placement.parked);
    assert_eq!(placement.bounds, state.bounds);
}
