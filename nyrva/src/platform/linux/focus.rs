use std::collections::HashMap;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ClientMessageData, ClientMessageEvent, ConnectionExt, EventMask, Window,
};

pub struct ProcMaps {
    pub ppid: HashMap<u32, u32>,
    pub name: HashMap<u32, String>,
}

fn parse_proc_stat(stat: &str) -> Option<(u32, u32, String)> {
    let open = stat.find('(')?;
    let close = stat.rfind(')')?;
    if close <= open { return None; }
    let pid = stat[..open].trim().parse::<u32>().ok()?;
    let name = stat[open + 1..close].to_lowercase();
    let mut fields = stat[close + 1..].split_whitespace();
    let _state = fields.next()?;
    let ppid = fields.next()?.parse::<u32>().ok()?;
    Some((pid, ppid, name))
}

pub fn proc_maps() -> ProcMaps {
    let mut maps = ProcMaps {
        ppid: HashMap::new(),
        name: HashMap::new(),
    };
    let Ok(entries) = std::fs::read_dir("/proc") else { return maps; };
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue; };
        let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) else { continue; };
        let Some((parsed_pid, ppid, name)) = parse_proc_stat(&stat) else { continue; };
        if parsed_pid != pid { continue; }
        maps.ppid.insert(pid, ppid);
        maps.name.insert(pid, name);
    }
    maps
}

fn intern_atom<C: Connection>(conn: &C, name: &[u8]) -> Option<Atom> {
    conn.intern_atom(false, name).ok()?.reply().ok().map(|r| r.atom)
}

fn window_pid<C: Connection>(conn: &C, window: Window, pid_atom: Atom) -> Option<u32> {
    let reply = conn
        .get_property(false, window, pid_atom, AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?;
    let mut values = reply.value32()?;
    values.next()
}

fn prefer_managed_windows(managed: Option<Vec<Window>>, fallback: Vec<Window>) -> Vec<Window> {
    match managed {
        Some(windows) if !windows.is_empty() => windows,
        _ => fallback,
    }
}

fn client_windows<C: Connection>(conn: &C, root: Window) -> Vec<Window> {
    let managed = intern_atom(conn, b"_NET_CLIENT_LIST").and_then(|client_list_atom| {
        let reply = conn
            .get_property(false, root, client_list_atom, AtomEnum::WINDOW, 0, 4096)
            .ok()?
            .reply()
            .ok()?;
        Some(reply.value32()?.collect::<Vec<_>>())
    });
    let fallback = conn
        .query_tree(root)
        .ok()
        .and_then(|cookie| cookie.reply().ok())
        .map(|tree| tree.children)
        .unwrap_or_default();
    prefer_managed_windows(managed, fallback)
}

pub fn fg_pid() -> u32 {
    let Ok((conn, screen_num)) = x11rb::connect(None) else { return 0; };
    let Some(screen) = conn.setup().roots.get(screen_num) else { return 0; };
    let Some(active_atom) = intern_atom(&conn, b"_NET_ACTIVE_WINDOW") else { return 0; };
    let Some(pid_atom) = intern_atom(&conn, b"_NET_WM_PID") else { return 0; };
    let Ok(cookie) = conn.get_property(false, screen.root, active_atom, AtomEnum::WINDOW, 0, 1) else { return 0; };
    let Ok(reply) = cookie.reply() else { return 0; };
    let Some(mut values) = reply.value32() else { return 0; };
    let Some(window) = values.next() else { return 0; };
    window_pid(&conn, window, pid_atom).unwrap_or(0)
}

pub fn chain_of(pid: u32, ppid: &HashMap<u32, u32>) -> Vec<u32> {
    let mut chain = vec![pid];
    let mut cur = pid;
    for _ in 0..8 {
        match ppid.get(&cur) {
            Some(&p) if p != 0 && !chain.contains(&p) => {
                chain.push(p);
                cur = p;
            }
            _ => break,
        }
    }
    chain
}

pub fn pid_hits_chain(pid: u32, chain: &[u32], maps: &ProcMaps) -> bool {
    chain.contains(&pid)
        || maps
            .ppid
            .get(&pid)
            .map(|p| chain.contains(p))
            .unwrap_or(false)
}

fn best_window_for_chain(windows: &[(Window, u32)], chain: &[u32], maps: &ProcMaps) -> Option<Window> {
    windows
        .iter()
        .filter_map(|(window, pid)| {
            let score = chain.iter().position(|p| p == pid).or_else(|| {
                maps.ppid
                    .get(pid)
                    .and_then(|parent| chain.iter().position(|p| p == parent))
            });
            score.map(|score| (score, *window))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, window)| window)
}

pub fn focus_terminal(pid: u32) -> bool {
    if pid == 0 { return false; }
    let maps = proc_maps();
    let chain = chain_of(pid, &maps.ppid);
    let Ok((conn, screen_num)) = x11rb::connect(None) else { return false; };
    let Some(screen) = conn.setup().roots.get(screen_num) else { return false; };
    let Some(pid_atom) = intern_atom(&conn, b"_NET_WM_PID") else { return false; };
    let Some(active_atom) = intern_atom(&conn, b"_NET_ACTIVE_WINDOW") else { return false; };
    let windows: Vec<(Window, u32)> = client_windows(&conn, screen.root)
        .into_iter()
        .filter_map(|window| window_pid(&conn, window, pid_atom).map(|pid| (window, pid)))
        .collect();
    let Some(window) = best_window_for_chain(&windows, &chain, &maps) else { return false; };

    // EWMH activation request. Source indication 1 means normal application.
    let event = ClientMessageEvent::new(
        32,
        window,
        active_atom,
        ClientMessageData::from([1, x11rb::CURRENT_TIME, 0, 0, 0]),
    );
    if conn
        .send_event(
            false,
            screen.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )
        .is_err()
    {
        return false;
    }
    conn.flush().is_ok()
}

// Claude Desktop Linux is intentionally outside M4. Claude Code sessions use focus_terminal().
pub fn focus_claude_desktop() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{best_window_for_chain, chain_of, parse_proc_stat, pid_hits_chain, prefer_managed_windows, ProcMaps};
    use std::collections::HashMap;

    #[test]
    fn chain_of_walks_parents_and_stops_at_zero() {
        let ppid = HashMap::from([(40, 30), (30, 20), (20, 0)]);
        assert_eq!(chain_of(40, &ppid), vec![40, 30, 20]);
    }

    #[test]
    fn chain_of_stops_on_cycles() {
        let ppid = HashMap::from([(40, 30), (30, 40)]);
        assert_eq!(chain_of(40, &ppid), vec![40, 30]);
    }

    #[test]
    fn pid_hits_chain_accepts_direct_and_parent_matches() {
        let maps = ProcMaps {
            ppid: HashMap::from([(99, 30)]),
            name: HashMap::new(),
        };
        let chain = vec![40, 30, 20];
        assert!(pid_hits_chain(40, &chain, &maps));
        assert!(pid_hits_chain(99, &chain, &maps));
        assert!(!pid_hits_chain(100, &chain, &maps));
    }

    #[test]
    fn proc_stat_parser_handles_process_names_with_spaces() {
        let stat = "4242 (claude worker) S 4000 1 1 0 -1 4194304";
        assert_eq!(
            parse_proc_stat(stat),
            Some((4242, 4000, "claude worker".to_string()))
        );
    }

    #[test]
    fn managed_window_list_is_preferred_over_root_tree_frames() {
        assert_eq!(
            prefer_managed_windows(Some(vec![101, 102]), vec![9001, 9002]),
            vec![101, 102]
        );
        assert_eq!(prefer_managed_windows(Some(vec![]), vec![9001]), vec![9001]);
        assert_eq!(prefer_managed_windows(None, vec![9002]), vec![9002]);
    }

    #[test]
    fn terminal_window_prefers_highest_matching_ancestor() {
        let maps = ProcMaps {
            ppid: HashMap::from([(99, 30)]),
            name: HashMap::new(),
        };
        let chain = vec![40, 30, 20];
        let windows = vec![(1001, 40), (1002, 99), (1003, 20), (1004, 777)];
        assert_eq!(best_window_for_chain(&windows, &chain, &maps), Some(1003));
    }
}
