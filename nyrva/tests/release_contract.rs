#[test]
fn mvp_release_contract_bundles_hook_and_publishes_installers() {
    let windows = include_str!("../tauri.windows.conf.json");
    let linux = include_str!("../tauri.linux.conf.json");
    let workflow = include_str!("../../.github/workflows/ci.yml");
    let windows_sidecar = include_str!("../../scripts/prepare-windows-sidecar.ps1");

    assert!(windows.contains("externalBin"));
    assert!(windows.contains("binaries/nyrva-hook"));
    assert!(linux.contains("externalBin"));
    assert!(linux.contains("binaries/nyrva-hook"));

    assert!(windows_sidecar.contains("nyrva-hook-$hostTriple.exe"));

    assert!(workflow.contains("name: nyrva-windows"));
    assert!(workflow.contains("target/release/bundle/nsis/*.exe"));
    assert!(workflow.contains("name: nyrva-linux"));
    assert!(workflow.contains("target/release/bundle/deb/*.deb"));
    assert!(workflow.contains("target/release/bundle/appimage/*.AppImage"));
}
