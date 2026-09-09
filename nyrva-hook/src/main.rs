//! nyrva-hook: the minimal client Claude Code's hooks call.
//! Duties: 1) report the event plus stdin JSON to the main app; 2) launch the main app if it is not running.
//! Iron rule: never block Claude Code — ~2 s total budget, and every failure exits 0 silently.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_PORT: u16 = 48666;
const MAX_STDIN: u64 = 256 * 1024;

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("nyrva").join("config.json"))
}

fn main_binary_name() -> &'static str {
    if cfg!(windows) { "nyrva.exe" } else { "nyrva" }
}

fn main() {
    let event = std::env::args().nth(1).unwrap_or_else(|| "ping".into());
    let mut body = String::new();
    let _ = std::io::stdin().take(MAX_STDIN).read_to_string(&mut body);
    let port = read_port();
    let ppid = parent_pid();
    if send(port, &event, ppid, &body).is_ok() { return; }
    spawn_main();
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(100));
        if send(port, &event, ppid, &body).is_ok() { return; }
    }
}

fn read_port() -> u16 {
    let Some(path) = config_path() else { return DEFAULT_PORT; };
    let Ok(txt) = std::fs::read_to_string(path) else { return DEFAULT_PORT; };
    if let Some(i) = txt.find("\"port\"") {
        let digits: String = txt[i + 6..].chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(p) = digits.parse() { return p; }
    }
    DEFAULT_PORT
}

fn parse_json_string_field(txt: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\"");
    let start = txt.find(&marker)? + marker.len();
    let after_colon = txt[start..].find(':')? + start + 1;
    let bytes = txt.as_bytes();
    let mut i = after_colon;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
    if bytes.get(i) != Some(&b'"') { return None; }
    i += 1;
    let mut out = String::new();
    while i < bytes.len() {
        match bytes[i] {
            b'"' => return Some(out),
            b'\\' => {
                i += 1;
                let escaped = *bytes.get(i)?;
                match escaped {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    _ => return None,
                }
            }
            b if b.is_ascii() => out.push(b as char),
            _ => {
                let rest = &txt[i..];
                let ch = rest.chars().next()?;
                out.push(ch);
                i += ch.len_utf8() - 1;
            }
        }
        i += 1;
    }
    None
}

fn launch_target_from_json(txt: &str) -> Option<PathBuf> {
    parse_json_string_field(txt, "launcher_path")
        .filter(|p| !p.trim().is_empty())
        .map(PathBuf::from)
}

fn persisted_launch_target() -> Option<PathBuf> {
    let path = config_path()?;
    let txt = std::fs::read_to_string(path).ok()?;
    launch_target_from_json(&txt)
}

fn send(port: u16, event: &str, ppid: u32, body: &str) -> std::io::Result<()> {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_millis(300))?;
    s.set_write_timeout(Some(Duration::from_millis(700)))?;
    s.set_read_timeout(Some(Duration::from_millis(700)))?;
    let req = format!("POST /event?e={}&ppid={} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", event, ppid, body.len(), body);
    s.write_all(req.as_bytes())?;
    let mut buf = [0u8; 64];
    let _ = s.read(&mut buf);
    Ok(())
}

fn spawn_main() {
    let exe = persisted_launch_target().or_else(|| {
        let me = std::env::current_exe().ok()?;
        Some(me.parent()?.join(main_binary_name()))
    });
    let Some(exe) = exe else { return; };
    if !exe.exists() { return; }
    let mut cmd = std::process::Command::new(exe);
    cmd.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
    }
    let _ = cmd.spawn();
}

#[cfg(windows)]
fn parent_pid() -> u32 {
    #[repr(C)] struct Pbi { exit_status: isize, peb: usize, affinity_mask: usize, base_priority: isize, unique_process_id: usize, inherited_from_unique_process_id: usize }
    extern "system" { fn NtQueryInformationProcess(handle: isize, class: u32, info: *mut Pbi, len: u32, ret_len: *mut u32) -> i32; }
    unsafe {
        let mut pbi = std::mem::zeroed::<Pbi>();
        let mut ret = 0u32;
        if NtQueryInformationProcess(-1, 0, &mut pbi, std::mem::size_of::<Pbi>() as u32, &mut ret) == 0 { return pbi.inherited_from_unique_process_id as u32; }
    }
    0
}

#[cfg(not(windows))]
fn parent_pid() -> u32 { std::os::unix::process::parent_id() }

#[cfg(test)]
mod tests {
    use super::{config_path, launch_target_from_json, main_binary_name};
    use std::path::{Path, PathBuf};

    #[test]
    fn main_binary_name_matches_platform() {
        #[cfg(windows)]
        assert_eq!(main_binary_name(), "nyrva.exe");
        #[cfg(target_os = "linux")]
        assert_eq!(main_binary_name(), "nyrva");
    }

    #[test]
    fn config_path_uses_nyrva_config_namespace() {
        let path = config_path().expect("config directory should be available in supported desktop environments");
        assert!(path.ends_with(Path::new("nyrva").join("config.json")));
    }

    #[test]
    fn persisted_launcher_path_is_read_from_config_json() {
        let json = r#"{"port":48666,"launcher_path":"/home/alice/Apps/Nyrva.AppImage"}"#;
        assert_eq!(
            launch_target_from_json(json),
            Some(PathBuf::from("/home/alice/Apps/Nyrva.AppImage"))
        );
    }

    #[test]
    fn missing_launcher_path_returns_none() {
        assert_eq!(launch_target_from_json(r#"{"port":48666}"#), None);
    }
}
