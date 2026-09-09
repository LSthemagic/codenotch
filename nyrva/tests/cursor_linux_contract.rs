#[test]
fn cursor_linux_parity_contracts_are_explicit() {
    let cursor = include_str!("../src/cursor.rs");
    let activity = include_str!("../src/activity.rs");

    assert!(
        cursor.contains("linux_config_dir_from"),
        "Cursor Linux config resolution must explicitly honor XDG_CONFIG_HOME"
    );
    assert!(
        cursor.contains("immutable_uri"),
        "Cursor immutable SQLite fallback must use a tested URI helper"
    );
    assert!(
        !cursor.contains("cannot locate %APPDATA%"),
        "Cursor diagnostics must not use Windows-only wording on Linux"
    );
    assert!(
        activity.contains("crate::cursor::store_url()"),
        "Cursor usage and activity must share the same state DB resolver"
    );
}
