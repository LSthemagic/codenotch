//! Local event server: receives nyrva-hook's POST /event?e=<event>&ppid=<pid>
//! with the Claude Code hook's stdin JSON as the body. Lenient parsing: no missing field is an error.

use crate::state::HookEvent;
use crate::AppState;
use std::io::Read;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "linux")]
fn start_linux_ack_scan(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let need = {
            let st = app.state::<AppState>();
            let store = st.store.lock().unwrap();
            store.has_done()
        };
        if !need { continue; }

        let fg = crate::focus::fg_pid();
        if fg == 0 { continue; }
        let maps = crate::focus::proc_maps();
        let changed = {
            let st = app.state::<AppState>();
            let mut store = st.store.lock().unwrap();
            store.ack_done(|session| {
                // Claude Desktop Linux is outside M4; hook-backed Claude Code sessions have a ppid.
                session.ppid != 0
                    && crate::focus::pid_hits_chain(
                        fg,
                        &crate::focus::chain_of(session.ppid, &maps.ppid),
                        &maps,
                    )
            })
        };
        if changed {
            crate::broadcast(&app);
        }
    });
}

pub fn start(app: AppHandle, port: u16) {
    #[cfg(target_os = "linux")]
    start_linux_ack_scan(app.clone());

    std::thread::spawn(move || {
        let server = match tiny_http::Server::http(("127.0.0.1", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[nyrva] failed to bind port {port}: {e} (is another instance running?)");
                return;
            }
        };
        for mut req in server.incoming_requests() {
            let url = req.url().to_string();
            let mut body = String::new();
            let _ = req
                .as_reader()
                .take(256 * 1024)
                .read_to_string(&mut body);
            if url.starts_with("/event") {
                let ev = parse(&url, &body);
                let state = app.state::<AppState>();
                let changed = {
                    let mut store = state.store.lock().unwrap();
                    store.apply(ev)
                };
                if changed {
                    crate::broadcast(&app);
                }
            }
            let _ = req.respond(tiny_http::Response::from_string("ok"));
        }
    });
}

fn query_param(url: &str, key: &str) -> String {
    let q = url.splitn(2, '?').nth(1).unwrap_or("");
    for pair in q.split('&') {
        let mut it = pair.splitn(2, '=');
        if it.next() == Some(key) {
            return it.next().unwrap_or("").to_string();
        }
    }
    String::new()
}

fn parse(url: &str, body: &str) -> HookEvent {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    // tool_input.command (Bash etc.) feeds the "last action" summary
    let tool_cmd = v
        .get("tool_input")
        .and_then(|t| t.get("command"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    HookEvent {
        e: query_param(url, "e"),
        session_id: {
            let id = s("session_id");
            if id.is_empty() { "unknown".into() } else { id }
        },
        ppid: query_param(url, "ppid").parse().unwrap_or(0),
        cwd: s("cwd"),
        prompt: s("prompt"),
        message: s("message"),
        tool_name: s("tool_name"),
        tool_cmd,
        model: s("model"),
        src: "hook",
    }
}
