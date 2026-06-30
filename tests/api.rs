use slint_webview::{
    CompositionTier, NavigationDecision, NavigationPolicy, WebViewBounds, WebViewCapabilities,
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
