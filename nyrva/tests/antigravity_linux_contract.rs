#[test]
fn antigravity_linux_parity_contracts_are_explicit() {
    let source = include_str!("../src/antigravity.rs");

    assert!(
        source.contains("linux_listening_ports_from_proc"),
        "Antigravity Linux bridge discovery must use /proc instead of requiring lsof"
    );
    assert!(
        source.contains("keyring::Entry::new"),
        "Antigravity Linux credentials must be borrowed from Secret Service"
    );
    assert!(
        !source.contains("run_hidden(\"lsof\""),
        "Antigravity Linux must not require lsof"
    );
}
