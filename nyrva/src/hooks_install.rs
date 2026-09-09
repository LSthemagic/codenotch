//! Merges the Nyrva hook into ~/.claude/settings.json without overwriting the user's own hooks.
//! Identification accepts both Nyrva and legacy Codenotch hook commands so upgrades can cleanly replace older entries.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const WIRING: &[(&str, bool, &str)] = &[
    ("SessionStart", false, "session_start"),
    ("UserPromptSubmit", false, "running"),
    ("PreToolUse", true, "running"),
    ("PostToolUse", true, "running"),
    ("Notification", false, "attention"),
    ("Stop", false, "done"),
    ("SessionEnd", false, "session_end"),
];

fn hook_binary_name() -> &'static str {
    if cfg!(windows) { "nyrva-hook.exe" } else { "nyrva-hook" }
}

fn bundled_hook_path(main_exe: &Path) -> PathBuf {
    main_exe
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(hook_binary_name())
}

fn persistent_hook_path_from(data_dir: &Path) -> PathBuf {
    data_dir.join("nyrva").join("bin").join(hook_binary_name())
}

fn hook_command(path: &Path, internal: &str) -> String {
    format!("\"{}\" {internal}", path.display())
}

#[cfg(target_os = "linux")]
fn install_hook_binary_to(source: &Path, destination: &Path) -> Result<PathBuf, String> {
    use std::os::unix::fs::PermissionsExt;

    let parent = destination.parent().ok_or("invalid persistent hook destination")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    std::fs::copy(source, destination).map_err(|e| e.to_string())?;
    let mut permissions = std::fs::metadata(destination).map_err(|e| e.to_string())?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(destination, permissions).map_err(|e| e.to_string())?;
    Ok(destination.to_path_buf())
}

fn settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

fn is_ours(entry: &Value) -> bool {
    entry["hooks"]
        .as_array()
        .map(|hs| {
            hs.iter().any(|h| {
                h["command"]
                    .as_str()
                    .map(|c| c.contains("nyrva-hook") || c.contains("codenotch-hook") || c.contains("eatbean-hook") || c.contains("pacman-hook"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn load(path: &PathBuf) -> Value {
    std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_else(|| json!({}))
}

fn backup_and_write(path: &PathBuf, root: &Value) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if path.exists() {
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = std::fs::copy(path, path.with_extension(format!("json.nyrva-bak-{ts}")));
    }
    let txt = serde_json::to_string_pretty(root).map_err(|e| e.to_string())?;
    std::fs::write(path, txt).map_err(|e| e.to_string())
}

pub fn is_installed() -> bool {
    settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| t.contains("nyrva-hook"))
        .unwrap_or(false)
}

pub fn install() -> Result<String, String> {
    let path = settings_path().ok_or("cannot find the user directory")?;
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let bundled = bundled_hook_path(&current_exe);
    if !bundled.exists() {
        return Err(format!("missing {}", bundled.display()));
    }

    // The persistent helper may outlive an AppImage mount. Record the executable that can
    // actually relaunch Nyrva: the outer APPIMAGE path when present, otherwise current_exe.
    let appimage = std::env::var("APPIMAGE").ok();
    let launch_target = crate::config::launcher_path_from(appimage.as_deref(), &current_exe);
    let mut cfg = crate::config::load();
    cfg.launcher_path = launch_target.to_string_lossy().into_owned();
    crate::config::save(&cfg);

    #[cfg(target_os = "linux")]
    let hook_exe = {
        let data_dir = dirs::data_local_dir().ok_or("cannot find the user data directory")?;
        let destination = persistent_hook_path_from(&data_dir);
        install_hook_binary_to(&bundled, &destination)?
    };
    #[cfg(not(target_os = "linux"))]
    let hook_exe = bundled;

    let mut root = load(&path);
    if !root.is_object() { root = json!({}); }
    if !root["hooks"].is_object() { root["hooks"] = json!({}); }

    for (event, need_matcher, internal) in WIRING {
        let arr = root["hooks"][*event].as_array().cloned().unwrap_or_default();
        let mut arr: Vec<Value> = arr.into_iter().filter(|e| !is_ours(e)).collect();
        let cmd = hook_command(&hook_exe, internal);
        let mut entry = json!({ "hooks": [{ "type": "command", "command": cmd, "timeout": 5 }] });
        if *need_matcher { entry["matcher"] = json!("*"); }
        arr.push(entry);
        root["hooks"][*event] = json!(arr);
    }

    backup_and_write(&path, &root)?;
    Ok(format!("wrote {} ({} events; hook {})", path.display(), WIRING.len(), hook_exe.display()))
}

pub fn uninstall() -> Result<String, String> {
    let path = settings_path().ok_or("cannot find the user directory")?;
    if !path.exists() { return Ok("settings.json does not exist, nothing to uninstall".into()); }
    let mut root = load(&path);
    let Some(hooks) = root["hooks"].as_object_mut() else { return Ok("no hooks configuration found".into()); };
    let mut removed = 0;
    for (_, v) in hooks.iter_mut() {
        if let Some(arr) = v.as_array() {
            let filtered: Vec<Value> = arr.iter().filter(|e| !is_ours(e)).cloned().collect();
            removed += arr.len() - filtered.len();
            *v = json!(filtered);
        }
    }
    backup_and_write(&path, &root)?;

    #[cfg(target_os = "linux")]
    if let Some(data_dir) = dirs::data_local_dir() {
        let helper = persistent_hook_path_from(&data_dir);
        if helper.exists() {
            let _ = std::fs::remove_file(helper);
        }
    }

    Ok(format!("removed {removed} Nyrva hook(s)"))
}

#[cfg(test)]
mod tests {
    use super::{hook_binary_name, hook_command};
    #[cfg(target_os = "linux")]
    use super::{bundled_hook_path, install_hook_binary_to, persistent_hook_path_from};
    use std::path::Path;
    #[cfg(target_os = "linux")]
    use std::path::PathBuf;

    #[test]
    fn hook_binary_name_matches_platform() {
        #[cfg(windows)]
        assert_eq!(hook_binary_name(), "nyrva-hook.exe");
        #[cfg(target_os = "linux")]
        assert_eq!(hook_binary_name(), "nyrva-hook");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn bundled_hook_is_next_to_main_executable() {
        let exe = Path::new("/tmp/.mount_Nyrva/usr/bin/nyrva");
        assert_eq!(bundled_hook_path(exe), PathBuf::from("/tmp/.mount_Nyrva/usr/bin/nyrva-hook"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn persistent_linux_hook_lives_under_user_data_dir() {
        let base = Path::new("/home/alice/.local/share");
        assert_eq!(
            persistent_hook_path_from(base),
            PathBuf::from("/home/alice/.local/share/nyrva/bin/nyrva-hook")
        );
    }

    #[test]
    fn hook_command_quotes_paths_with_spaces() {
        let path = Path::new("/home/alice/Nyrva Data/bin/nyrva-hook");
        assert_eq!(hook_command(path, "running"), "\"/home/alice/Nyrva Data/bin/nyrva-hook\" running");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn persistent_hook_copy_is_executable() {
        use std::os::unix::fs::PermissionsExt;
        let unique = format!(
            "nyrva-hook-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let source = root.join("source-hook");
        let destination = root.join("data/nyrva/bin/nyrva-hook");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&source, b"hook-bytes").unwrap();

        let installed = install_hook_binary_to(&source, &destination).unwrap();
        assert_eq!(installed, destination);
        assert_eq!(std::fs::read(&installed).unwrap(), b"hook-bytes");
        assert_ne!(std::fs::metadata(&installed).unwrap().permissions().mode() & 0o111, 0);

        let _ = std::fs::remove_dir_all(root);
    }
}
