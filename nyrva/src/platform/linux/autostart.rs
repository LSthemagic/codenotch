#[cfg(test)]
mod tests {
    use super::{desktop_entry, disable_at, enable_at, is_enabled_at};
    use std::path::Path;

    fn temp_entry() -> std::path::PathBuf {
        let unique = format!(
            "nyrva-autostart-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        std::env::temp_dir().join(unique).join("autostart").join("nyrva.desktop")
    }

    #[test]
    fn renders_nyrva_desktop_entry_and_escapes_exec_path() {
        let s = desktop_entry(Path::new(r#"/opt/Nyrva $App/nyrva"#));
        assert!(s.contains("Name=Nyrva"));
        assert!(s.contains("Terminal=false"));
        assert!(s.contains(r#"Exec=\"/opt/Nyrva \$App/nyrva\" --silent"#));
    }

    #[test]
    fn enable_and_disable_only_the_nyrva_entry() {
        let path = temp_entry();
        enable_at(&path, Path::new("/opt/nyrva")).unwrap();
        assert!(is_enabled_at(&path));
        assert!(path.exists());

        disable_at(&path).unwrap();
        assert!(!path.exists());
        assert!(!is_enabled_at(&path));

        // Missing file is idempotent success.
        disable_at(&path).unwrap();
        if let Some(root) = path.parent().and_then(|p| p.parent()) {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}
