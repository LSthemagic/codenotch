#[test]
fn activity_uses_shared_codex_paths() {
    let source = include_str!("../src/activity.rs");
    assert!(
        source.contains("crate::codex::codex_paths()"),
        "activity probe must consume the shared Codex path resolver"
    );
    assert!(
        !source.contains("home.join(\".codex\")"),
        "activity probe must not hard-code ~/.codex"
    );
}
