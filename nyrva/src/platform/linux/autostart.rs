use std::path::{Path, PathBuf};

const FILE_NAME: &str = "nyrva.desktop";

fn autostart_path() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(xdg).join("autostart").join(FILE_NAME));
    }
    dirs::config_dir().map(|d| d.join("autostart").join(FILE_NAME))
}

fn escape_exec_arg(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let mut out = String::with_capacity(raw.len() + 2);
    out.push('"');
    for ch in raw.chars() {
        match ch {
            '\\' | '"' | '`' | '$' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Nyrva\nExec={} --silent\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        escape_exec_arg(exe)
    )
}

fn is_enabled_at(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .map(|s| {
            s.lines().any(|l| l.trim() == "Name=Nyrva")
                && s.lines().any(|l| l.starts_with("Exec=") && l.to_ascii_lowercase().contains("nyrva"))
        })
        .unwrap_or(false)
}

fn enable_at(path: &Path, exe: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or("invalid autostart path")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    std::fs::write(path, desktop_entry(exe)).map_err(|e| e.to_string())
}

fn disable_at(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn is_enabled() -> bool {
    autostart_path().map(|p| is_enabled_at(&p)).unwrap_or(false)
}

pub fn enable() -> Result<String, String> {
    let path = autostart_path().ok_or("cannot find the user config directory")?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    enable_at(&path, &exe)?;
    Ok(format!("start at sign-in enabled ({})", path.display()))
}

pub fn disable() -> Result<String, String> {
    let path = autostart_path().ok_or("cannot find the user config directory")?;
    disable_at(&path)?;
    Ok("start at sign-in disabled".into())
}

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
        assert!(s.contains(r#"Exec="/opt/Nyrva \$App/nyrva" --silent"#));
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

        disable_at(&path).unwrap();
        if let Some(root) = path.parent().and_then(|p| p.parent()) {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}
